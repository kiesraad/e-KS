#![feature(rustc_private)]
#![warn(unused_extern_crates)]

//! A Dylint library that rejects cyclic dependencies between components.

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use rustc_errors::DiagDecorator;
use rustc_hir::{HirId, ItemKind, Node, Path, def::Res, def_id::DefId};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::{FileName, Span, def_id::LOCAL_CRATE};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    path::{Component as PathComponent, Path as FilePath, PathBuf},
};

dylint_linting::impl_late_lint! {
    /// ### What it does
    ///
    /// Rejects cyclic dependencies between components, a component being a
    /// top-level directory under `src/`. Cycles are of any length: `A -> B -> A`
    /// and `A -> B -> C -> A` are both rejected. One finding is emitted per group
    /// of components that all reach each other, heaviest first, naming the
    /// cheapest edge in the group to remove. Every dependency between components
    /// is reported as a note as well, cyclic or not, so the shape of the graph is
    /// visible even when nothing is rejected.
    ///
    /// ### Why is this bad?
    ///
    /// Cyclic dependencies couple components in both directions, making them
    /// harder to understand, test, and change independently.
    ///
    /// ### Known problems
    ///
    /// Dependencies come from resolved paths, so calls reached through the type
    /// of a receiver (`value.method()`) do not count. References produced by
    /// macro expansion are attributed to neither side.
    pub MODULE_DEPENDENCY_CYCLES,
    Deny,
    "cyclic dependencies between components",
    ModuleDependencyCycles::default()
}

/// The component holding the files that sit directly in `src/`, which belong to
/// no component directory of their own.
const ROOT: &str = "root";

/// Directories under `src/` that are not production code.
const EXCLUDED_DIRECTORIES: &[&str] = &["bin", "fixtures"];

#[derive(Default)]
struct ModuleDependencyCycles {
    graph: ComponentGraph,
    /// `check_path` fires for every resolved path in the crate, so the mapping
    /// from source file to component is memoised.
    components: BTreeMap<PathBuf, Option<Located>>,
}

impl<'tcx> LateLintPass<'tcx> for ModuleDependencyCycles {
    fn check_path(&mut self, cx: &LateContext<'tcx>, path: &Path<'tcx>, _: HirId) {
        // A test-mode compilation also contains every `#[cfg(test)]` block, which
        // is not production code and must not create dependencies.
        if cx.sess().is_test_crate() || path.span.from_expansion() {
            return;
        }

        let Res::Def(_, def_id) = path.res else {
            return;
        };
        if !def_id.is_local() {
            return;
        }

        let Some(source) = self.locate(cx, path.span) else {
            return;
        };
        let Some(target) = self.locate(cx, definition_span(cx, def_id)) else {
            return;
        };
        if source.component == target.component {
            return;
        }

        self.graph.add_dependency(
            source.component,
            target.component,
            Dependency {
                file: source.file,
                item: cx.tcx.def_path_str(def_id),
            },
            path.span,
        );
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        if let Some(summary) = self.graph.summary() {
            cx.sess().dcx().note(format!(
                "component dependencies in `{}`:\n{summary}",
                cx.tcx.crate_name(LOCAL_CRATE),
            ));
        }

        for cycle in self.graph.cycles() {
            let Cycle {
                components,
                from,
                to,
                remove,
                back,
            } = cycle;

            let rest = if components.len() == 2 {
                "the other direction"
            } else {
                "the rest of the cycle"
            };
            let message = format!(
                "cyclic dependency between components {}: {} to remove against {} in {rest}",
                component_list(&components),
                dependency_count(remove.len()),
                dependency_count(back),
            );
            let summary = format!(
                "`{from}` -> `{to}` is the cheapest edge in the cycle, and {rest} has {}, \
                 so breaking the cycle here is the cheaper direction",
                dependency_count(back),
            );
            let notes = remove
                .iter()
                .map(|(dependency, span)| {
                    (
                        *span,
                        format!("`{}` depends on `{}`", dependency.file, dependency.item),
                    )
                })
                .collect::<Vec<_>>();

            cx.emit_span_lint(
                MODULE_DEPENDENCY_CYCLES,
                remove
                    .first()
                    .map_or(rustc_span::DUMMY_SP, |(_, span)| *span),
                DiagDecorator(move |diag| {
                    diag.primary_message(message);
                    for (span, note) in notes {
                        diag.span_note(span, note);
                    }
                    diag.note(summary);
                }),
            );
        }
    }
}

/// The components of a cycle as `` `a` and `b` `` or `` `a`, `b` and `c` ``.
fn component_list(components: &[String]) -> String {
    let quoted = components
        .iter()
        .map(|component| format!("`{component}`"))
        .collect::<Vec<_>>();

    match quoted.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

fn dependency_count(count: usize) -> String {
    if count == 1 {
        "1 dependency".to_owned()
    } else {
        format!("{count} dependencies")
    }
}

/// Where an item is defined, for the purpose of owning a dependency on it.
fn definition_span(cx: &LateContext<'_>, def_id: DefId) -> Span {
    // `mod foo;` is declared in the parent file, but depending on the module
    // means depending on the file it names, which is its inner span.
    if let Some(local) = def_id.as_local()
        && let Node::Item(item) = cx.tcx.hir_node_by_def_id(local)
        && let ItemKind::Mod(_, module) = item.kind
    {
        return module.spans.inner_span;
    }
    cx.tcx.def_span(def_id).source_callsite()
}

/// A file that is part of the analysed source, and the component owning it.
#[derive(Clone)]
struct Located {
    component: String,
    file: String,
}

impl ModuleDependencyCycles {
    fn locate(&mut self, cx: &LateContext<'_>, span: Span) -> Option<Located> {
        let name = cx.sess().source_map().span_to_filename(span);
        let FileName::Real(real) = &name else {
            return None;
        };
        let path = workspace_relative(real.local_path()?);

        self.components
            .entry(path)
            .or_insert_with_key(|path| {
                Some(Located {
                    component: component_of(path)?,
                    file: path.to_string_lossy().into_owned(),
                })
            })
            .clone()
    }
}

/// Cargo runs rustc from the workspace root, so file names are already relative
/// to it. Absolute names are made relative so both forms map onto a component.
fn workspace_relative(path: &FilePath) -> PathBuf {
    static WORKSPACE_ROOT: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

    WORKSPACE_ROOT
        .get_or_init(|| std::env::current_dir().ok())
        .as_deref()
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path)
        .to_path_buf()
}

/// Every directory in `src/` is a component and the files directly in it form
/// [`ROOT`]. Everything else has no component and is not analysed.
fn component_of(path: &FilePath) -> Option<String> {
    if is_test_code(path) {
        return None;
    }

    let mut segments = path.components();
    if segments.next()? != PathComponent::Normal("src".as_ref()) {
        return None;
    }

    let PathComponent::Normal(directory) = segments.next()? else {
        return None;
    };
    if segments.next().is_none() {
        return Some(ROOT.to_owned());
    }

    let directory = directory.to_str()?;
    if EXCLUDED_DIRECTORIES.contains(&directory) {
        return None;
    }
    Some(format!("src/{directory}"))
}

fn is_test_code(path: &FilePath) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };

    name.contains("test")
}

/// One referenced item, counted once per file that references it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Dependency {
    file: String,
    item: String,
}

struct Cycle {
    /// Every component in the cycle, sorted.
    components: Vec<String>,
    /// The cheapest edge in the cycle, the one to remove.
    from: String,
    to: String,
    /// The dependencies on that edge.
    remove: Vec<(Dependency, Span)>,
    /// How many dependencies the rest of the cycle carries.
    back: usize,
}

#[derive(Default)]
struct ComponentGraph {
    /// `from -> to -> dependency -> first span that introduced it`.
    edges: BTreeMap<String, BTreeMap<String, BTreeMap<Dependency, Span>>>,
}

impl ComponentGraph {
    fn add_dependency(&mut self, from: String, to: String, dependency: Dependency, span: Span) {
        self.edges
            .entry(from)
            .or_default()
            .entry(to)
            .or_default()
            .entry(dependency)
            .or_insert(span);
    }

    fn weight(&self, from: &str, to: &str) -> usize {
        self.edges
            .get(from)
            .and_then(|targets| targets.get(to))
            .map_or(0, BTreeMap::len)
    }

    fn dependencies(&self, from: &str, to: &str) -> Vec<(Dependency, Span)> {
        self.edges
            .get(from)
            .and_then(|targets| targets.get(to))
            .map(|dependencies| {
                dependencies
                    .iter()
                    .map(|(dependency, span)| (dependency.clone(), *span))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Every component that appears in the graph, on either side of an edge.
    fn components(&self) -> BTreeSet<&str> {
        self.edges
            .iter()
            .flat_map(|(from, targets)| {
                std::iter::once(from.as_str()).chain(targets.keys().map(String::as_str))
            })
            .collect()
    }

    /// For every component, the components it reaches over one or more edges.
    fn reachable(&self) -> BTreeMap<&str, BTreeSet<&str>> {
        let mut reachable: BTreeMap<&str, BTreeSet<&str>> = self
            .edges
            .iter()
            .map(|(from, targets)| (from.as_str(), targets.keys().map(String::as_str).collect()))
            .collect();

        // The graph has a handful of nodes, so growing every set until nothing
        // changes is cheap enough and needs no bookkeeping.
        let mut grown = true;
        while grown {
            grown = false;
            for from in self.components() {
                let indirect = reachable
                    .get(from)
                    .into_iter()
                    .flatten()
                    .filter_map(|via| reachable.get(via))
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                let targets = reachable.entry(from).or_default();
                for target in indirect {
                    grown |= targets.insert(target);
                }
            }
        }
        reachable
    }

    /// The groups of components that all reach each other, so every edge within
    /// a group is part of a cycle. Components outside such a group are left out.
    fn cyclic_groups(&self) -> Vec<Vec<&str>> {
        let reachable = self.reachable();
        let reaches =
            |from: &str, to: &str| reachable.get(from).is_some_and(|set| set.contains(to));

        let mut grouped = BTreeSet::new();
        let mut groups = Vec::new();
        for component in self.components() {
            // A component never depends on itself, so reaching itself means it
            // does so over a cycle through at least one other component.
            if grouped.contains(component) || !reaches(component, component) {
                continue;
            }
            let group = self
                .components()
                .into_iter()
                .filter(|other| reaches(component, other) && reaches(other, component))
                .collect::<Vec<_>>();
            grouped.extend(group.iter().copied());
            groups.push(group);
        }
        groups
    }

    /// Every edge in the graph, one `from -> to` per line and the cyclic ones
    /// marked as such. `None` when no component depends on another.
    fn summary(&self) -> Option<String> {
        let cyclic = self
            .cyclic_groups()
            .into_iter()
            .flat_map(|group| {
                group
                    .iter()
                    .flat_map(|from| group.iter().map(move |to| (*from, *to)))
                    .collect::<Vec<_>>()
            })
            .collect::<BTreeSet<_>>();
        let cyclic = &cyclic;

        let lines = self
            .edges
            .iter()
            .flat_map(|(from, targets)| {
                targets.iter().map(move |(to, dependencies)| {
                    let cyclic = if cyclic.contains(&(from.as_str(), to.as_str())) {
                        ", cyclic"
                    } else {
                        ""
                    };
                    format!(
                        "  {from} -> {to} ({}{cyclic})",
                        dependency_count(dependencies.len()),
                    )
                })
            })
            .collect::<Vec<_>>();

        (!lines.is_empty()).then(|| lines.join("\n"))
    }

    /// One cycle per group of components that all reach each other, the group
    /// with the most dependencies to remove first.
    fn cycles(&self) -> Vec<Cycle> {
        let mut cycles = self
            .cyclic_groups()
            .into_iter()
            .filter_map(|group| {
                let edges = group
                    .iter()
                    .flat_map(|from| group.iter().map(move |to| (*from, *to)))
                    .map(|(from, to)| (self.weight(from, to), from, to))
                    .filter(|(weight, ..)| *weight > 0)
                    .collect::<Vec<_>>();

                // Breaking the cheapest edge is the cheapest way into a cycle;
                // a group needing more than one edge removed reports again on
                // the next run. Ties break on the names, to stay deterministic.
                let &(_, from, to) = edges.iter().min()?;
                let back = edges
                    .iter()
                    .filter(|(_, edge_from, edge_to)| (*edge_from, *edge_to) != (from, to))
                    .map(|(weight, ..)| weight)
                    .sum();

                Some(Cycle {
                    components: group
                        .iter()
                        .map(|&component| component.to_owned())
                        .collect(),
                    from: from.to_owned(),
                    to: to.to_owned(),
                    remove: self.dependencies(from, to),
                    back,
                })
            })
            .collect::<Vec<_>>();

        cycles.sort_by(|left, right| {
            right
                .remove
                .len()
                .cmp(&left.remove.len())
                .then_with(|| left.from.cmp(&right.from))
                .then_with(|| left.to.cmp(&right.to))
        });
        cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_span::DUMMY_SP;

    fn graph(edges: &[(&str, &str, &str)]) -> ComponentGraph {
        let mut graph = ComponentGraph::default();
        for (from, to, item) in edges {
            graph.add_dependency(
                (*from).to_owned(),
                (*to).to_owned(),
                Dependency {
                    file: format!("{from}/file.rs"),
                    item: (*item).to_owned(),
                },
                DUMMY_SP,
            );
        }
        graph
    }

    #[test]
    fn reports_a_cyclic_pair_but_not_an_acyclic_dependency() {
        let cycles = graph(&[
            ("src/a", "src/b", "B"),
            ("src/b", "src/a", "A1"),
            ("src/b", "src/a", "A2"),
            ("src/b", "src/c", "C"),
        ])
        .cycles();

        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].components, ["src/a", "src/b"]);
        assert_eq!(cycles[0].from, "src/a");
        assert_eq!(cycles[0].to, "src/b");
        assert_eq!(cycles[0].remove.len(), 1);
        assert_eq!(cycles[0].back, 2);
    }

    #[test]
    fn reports_a_cycle_spanning_three_components() {
        let cycles = graph(&[
            ("src/a", "src/b", "B1"),
            ("src/a", "src/b", "B2"),
            ("src/b", "src/c", "C"),
            ("src/c", "src/a", "A1"),
            ("src/c", "src/a", "A2"),
            ("src/c", "src/d", "D"),
        ])
        .cycles();

        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].components, ["src/a", "src/b", "src/c"]);
        assert_eq!(cycles[0].from, "src/b");
        assert_eq!(cycles[0].to, "src/c");
        assert_eq!(cycles[0].remove.len(), 1);
        assert_eq!(cycles[0].back, 4);
    }

    #[test]
    fn groups_overlapping_cycles_into_one_finding() {
        let cycles = graph(&[
            ("src/a", "src/b", "B"),
            ("src/b", "src/a", "A1"),
            ("src/b", "src/c", "C"),
            ("src/c", "src/a", "A2"),
        ])
        .cycles();

        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].components, ["src/a", "src/b", "src/c"]);
    }

    #[test]
    fn does_not_report_an_acyclic_graph() {
        let cycles = graph(&[
            ("src/a", "src/b", "B"),
            ("src/a", "src/c", "C"),
            ("src/b", "src/d", "D1"),
            ("src/c", "src/d", "D2"),
        ])
        .cycles();

        assert!(cycles.is_empty());
    }

    #[test]
    fn counts_a_referenced_item_once_per_referencing_file() {
        let mut graph = graph(&[("src/a", "src/b", "B")]);
        graph.add_dependency(
            "src/a".to_owned(),
            "src/b".to_owned(),
            Dependency {
                file: "src/a/file.rs".to_owned(),
                item: "B".to_owned(),
            },
            DUMMY_SP,
        );

        assert_eq!(graph.weight("src/a", "src/b"), 1);
    }

    #[test]
    fn summarises_every_edge_and_marks_the_cyclic_ones() {
        let summary = graph(&[
            ("src/a", "src/b", "B"),
            ("src/b", "src/a", "A1"),
            ("src/b", "src/a", "A2"),
            ("src/b", "src/c", "C"),
            ("src/c", "src/d", "D"),
            ("src/d", "src/e", "E"),
            ("src/e", "src/c", "C"),
        ])
        .summary();

        assert_eq!(
            summary.as_deref(),
            Some(
                "  src/a -> src/b (1 dependency, cyclic)\n  \
                 src/b -> src/a (2 dependencies, cyclic)\n  \
                 src/b -> src/c (1 dependency)\n  \
                 src/c -> src/d (1 dependency, cyclic)\n  \
                 src/d -> src/e (1 dependency, cyclic)\n  \
                 src/e -> src/c (1 dependency, cyclic)"
            )
        );
        assert_eq!(ComponentGraph::default().summary(), None);
    }

    #[test]
    fn heavier_cycles_come_first() {
        let cycles = graph(&[
            ("src/a", "src/b", "B"),
            ("src/b", "src/a", "A"),
            ("src/c", "src/d", "D1"),
            ("src/c", "src/d", "D2"),
            ("src/d", "src/c", "C1"),
            ("src/d", "src/c", "C2"),
        ])
        .cycles();

        assert_eq!(
            cycles
                .iter()
                .map(|cycle| (cycle.from.as_str(), cycle.remove.len()))
                .collect::<Vec<_>>(),
            vec![("src/c", 2), ("src/a", 1)]
        );
    }

    #[test]
    fn maps_files_onto_components() {
        let component = |path: &str| component_of(FilePath::new(path));

        assert_eq!(component("src/pg/store/mod.rs").as_deref(), Some("src/pg"));
        assert_eq!(component("src/lib.rs").as_deref(), Some(ROOT));
        assert_eq!(component("src/crypto.rs").as_deref(), Some(ROOT));
        assert_eq!(component("auth-service/src/lib.rs"), None);
        assert_eq!(component("src/bin/eks.rs"), None);
        assert_eq!(component("src/fixtures/persons.rs"), None);
        assert_eq!(component("src/core/locale_tests.rs"), None);
        assert_eq!(component("src/utils/test_utils.rs"), None);
    }
}
