//! Read-only, namespace-aware XML DOM over [`roxmltree`], plus scoped traversal
//! helpers used by the SAML validators.
//!
//! [`roxmltree`] is a vetted, fuzzed, read-only DOM that resolves XML namespaces
//! and exposes each node's byte range in the source. We wrap it so the rest of
//! the crate keeps an index-based (`Document` + [`NodeId`]) surface: `parse`,
//! `document_element`, `get_attribute` (by local name), `local_name`,
//! `first_element_child`, [`inner_text`] (recursive, unescaped), and
//! [`Document::node_source`] (the raw source bytes of a node).
//!
//! Element lookups ([`find_child`], [`find_descendant`], …) match by
//! `(namespace-URI, local-name)`, so a `<saml:Issuer>` is only found when `saml`
//! resolves to the SAML assertion namespace, never by bare local name. This
//! avoids namespace-confusion attacks.
//!
//! SECURITY (XML Signature Wrapping): roxmltree excludes comments and processing
//! instructions from the element tree, and exclusive-c14n (used by the signature
//! backend) excludes them from the digest. The whole signed document is parsed
//! exactly once and the validators navigate that single tree, so an element
//! forged inside a comment is invisible to both extraction and the signature.

use std::ops::Range;

/// Index of a node within a [`Document`].
pub type NodeId = roxmltree::NodeId;

/// A parsed, namespace-resolved XML document borrowing its source.
pub struct Document<'a> {
    inner: roxmltree::Document<'a>,
}

/// Opaque XML parse error (`Display`), convertible into `AuthError`.
#[derive(Debug)]
pub struct XmlError(String);

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for XmlError {}

/// Cap on parsed node count, bounding memory from an oversized document; a
/// legitimate SAML message has at most a few thousand nodes. (DTDs are already
/// rejected: `ParsingOptions::allow_dtd` defaults to `false`.)
const NODE_LIMIT: u32 = 100_000;

/// Parse an XML string into a namespace-resolved [`Document`]. Errors on
/// malformed XML, an undeclared namespace prefix, empty input, or more than
/// `NODE_LIMIT` nodes.
pub fn parse(xml: &str) -> Result<Document<'_>, XmlError> {
    let opts = roxmltree::ParsingOptions {
        nodes_limit: NODE_LIMIT,
        ..Default::default()
    };
    roxmltree::Document::parse_with_options(xml, opts)
        .map(|inner| Document { inner })
        .map_err(|e| XmlError(e.to_string()))
}

fn node_matches(node: roxmltree::Node<'_, '_>, ns: &str, local: &str) -> bool {
    node.is_element() && node.tag_name().name() == local && node.tag_name().namespace() == Some(ns)
}

impl<'a> Document<'a> {
    fn node(&self, id: NodeId) -> Option<roxmltree::Node<'_, 'a>> {
        self.inner.get_node(id)
    }

    /// The root (document) element. Guaranteed to exist: [`parse`] rejects a
    /// document without one.
    pub fn document_element(&self) -> NodeId {
        self.inner.root_element().id()
    }

    /// The local (unprefixed) name of element `id`, or `None` for a non-element.
    pub fn local_name(&self, id: NodeId) -> Option<&str> {
        let n = self.node(id)?;
        n.is_element().then(|| n.tag_name().name())
    }

    /// The value of attribute `name` (matched by local name) on element `id`.
    pub fn get_attribute(&self, id: NodeId, name: &str) -> Option<&str> {
        self.node(id)?
            .attributes()
            .find(|a| a.name() == name)
            .map(|a| a.value())
    }

    /// The raw source bytes of node `id` (opening `<` through closing `>`),
    /// exactly as they appear in the parsed input.
    pub fn node_source(&self, id: NodeId) -> Option<&str> {
        let range: Range<usize> = self.node(id)?.range();
        self.inner.input_text().get(range)
    }

    /// The first child element of `id` (skipping text/comment nodes), if any.
    pub fn first_element_child(&self, id: NodeId) -> Option<NodeId> {
        self.node(id)?
            .children()
            .find(|c| c.is_element())
            .map(|c| c.id())
    }
}

/// All text under `id`, concatenated depth-first and unescaped.
pub fn inner_text(doc: &Document, id: NodeId) -> String {
    match doc.node(id) {
        Some(n) => n
            .descendants()
            .filter_map(|d| d.is_text().then(|| d.text()).flatten())
            .collect(),
        None => String::new(),
    }
}

/// Find the first direct child element matching `(ns, local_name)`.
pub fn find_child(doc: &Document, id: NodeId, ns: &str, local_name: &str) -> Option<NodeId> {
    doc.node(id)?
        .children()
        .find(|c| node_matches(*c, ns, local_name))
        .map(|n| n.id())
}

/// Collect all direct child elements matching `(ns, local_name)`, in document order.
pub fn children_by_tag(doc: &Document, id: NodeId, ns: &str, local_name: &str) -> Vec<NodeId> {
    match doc.node(id) {
        Some(n) => n
            .children()
            .filter(|c| node_matches(*c, ns, local_name))
            .map(|c| c.id())
            .collect(),
        None => Vec::new(),
    }
}

/// Find the first descendant element (excluding `id` itself) matching
/// `(ns, local_name)`, in document order.
pub fn find_descendant(doc: &Document, id: NodeId, ns: &str, local_name: &str) -> Option<NodeId> {
    doc.node(id)?
        .descendants()
        .skip(1)
        .find(|d| node_matches(*d, ns, local_name))
        .map(|n| n.id())
}

/// Find all descendant elements (excluding `id` itself) matching `(ns, local_name)`.
pub fn descendants_by_tag(doc: &Document, id: NodeId, ns: &str, local_name: &str) -> Vec<NodeId> {
    match doc.node(id) {
        Some(n) => n
            .descendants()
            .skip(1)
            .filter(|d| node_matches(*d, ns, local_name))
            .map(|d| d.id())
            .collect(),
        None => Vec::new(),
    }
}

/// A `(namespace-URI, local-name)` element tag, for the pruned lookups below.
pub type Tag<'a> = (&'a str, &'a str);

/// Like [`find_descendant`], but never descends into a subtree whose root
/// element matches `prune`.
///
/// Used by assertion validation to read claims from the outer RD Assertion while
/// skipping the `<saml:Advice>` evidence subtree (the AD assertions), which
/// carries its own Recipient / InResponseTo / scheme-specific LoA (eID §7.6.3).
pub fn find_descendant_pruned(doc: &Document, id: NodeId, tag: Tag, prune: Tag) -> Option<NodeId> {
    descendants_by_tag_pruned(doc, id, tag, prune)
        .into_iter()
        .next()
}

/// Like [`descendants_by_tag`], but skips any subtree rooted at an element
/// matching `prune`. See [`find_descendant_pruned`].
pub fn descendants_by_tag_pruned(doc: &Document, id: NodeId, tag: Tag, prune: Tag) -> Vec<NodeId> {
    walk_pruned(doc, id, prune)
        .into_iter()
        .filter(|&n| {
            doc.node(n)
                .is_some_and(|node| node_matches(node, tag.0, tag.1))
        })
        .collect()
}

/// Pre-order list of descendant element ids under `id` (excluding `id` itself),
/// skipping any subtree rooted at a `prune` element.
fn walk_pruned(doc: &Document, id: NodeId, prune: Tag) -> Vec<NodeId> {
    let mut out = Vec::new();
    let Some(root) = doc.node(id) else {
        return out;
    };
    for child in root.children() {
        collect_pruned(child, prune, &mut out);
    }
    out
}

fn collect_pruned(node: roxmltree::Node<'_, '_>, prune: Tag, out: &mut Vec<NodeId>) {
    if !node.is_element() {
        return;
    }
    if node_matches(node, prune.0, prune.1) {
        return; // prune this subtree entirely
    }
    out.push(node.id());
    for child in node.children() {
        collect_pruned(child, prune, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saml::constants::{NS_DSIG, NS_SAML, NS_SAMLP};

    #[test]
    fn inner_text_concatenates_recursively() {
        let doc = parse(r#"<r xmlns="urn:x">hello <b>world</b></r>"#).unwrap();
        let root = doc.document_element();
        assert_eq!(inner_text(&doc, root), "hello world");
    }

    #[test]
    fn find_descendant_matches_by_namespace_not_just_local_name() {
        // Two elements share the local name "Issuer" but live in different
        // namespaces; the lookup must only match the requested namespace.
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" xmlns:saml="{NS_SAML}"><other:Issuer xmlns:other="urn:other">WRONG</other:Issuer><saml:Issuer>RIGHT</saml:Issuer></samlp:Response>"#
        );
        let doc = parse(&xml).unwrap();
        let root = doc.document_element();
        let issuer = find_descendant(&doc, root, NS_SAML, "Issuer").unwrap();
        assert_eq!(inner_text(&doc, issuer), "RIGHT");
        assert!(find_descendant(&doc, root, "urn:other", "Issuer").is_some());
        assert!(find_descendant(&doc, root, NS_SAMLP, "Issuer").is_none());
    }

    #[test]
    fn children_by_tag_only_returns_direct_children() {
        // A Signature directly on the root plus one nested in a child: only the
        // direct child is returned (the scoping that keeps ArtifactResponse
        // verification from choking on the nested, differently-signed Assertion).
        let xml = format!(
            r#"<Root xmlns="{NS_DSIG}"><Signature>outer</Signature><Child><Signature>inner</Signature></Child></Root>"#
        );
        let doc = parse(&xml).unwrap();
        let root = doc.document_element();
        let sigs = children_by_tag(&doc, root, NS_DSIG, "Signature");
        assert_eq!(sigs.len(), 1);
        assert_eq!(inner_text(&doc, sigs[0]), "outer");
        // descendants_by_tag, by contrast, finds both.
        assert_eq!(
            descendants_by_tag(&doc, root, NS_DSIG, "Signature").len(),
            2
        );
    }

    #[test]
    fn attribute_access_by_local_name() {
        let doc = parse(r#"<el xmlns="urn:x" foo="bar" baz="qux"/>"#).unwrap();
        let root = doc.document_element();
        assert_eq!(doc.get_attribute(root, "foo"), Some("bar"));
        assert_eq!(doc.get_attribute(root, "baz"), Some("qux"));
        assert_eq!(doc.get_attribute(root, "missing"), None);
    }

    #[test]
    fn node_source_returns_exact_node_bytes() {
        // `node_source` must return the raw source slice of a node, from its
        // opening `<` to the end of its closing tag, bytes intact. The
        // EncryptedID decryption / signature paths depend on this.
        let xml =
            r#"<root xmlns="urn:r"><a>x</a><enc xmlns="urn:e"><data>cipher</data></enc></root>"#;
        let doc = parse(xml).unwrap();
        let root = doc.document_element();
        let enc = find_descendant(&doc, root, "urn:e", "enc").unwrap();
        assert_eq!(
            doc.node_source(enc),
            Some(r#"<enc xmlns="urn:e"><data>cipher</data></enc>"#)
        );
    }

    #[test]
    fn inner_text_unescapes_entities() {
        let doc = parse(r#"<r xmlns="urn:x">a &amp; b &lt;c&gt;</r>"#).unwrap();
        let root = doc.document_element();
        assert_eq!(inner_text(&doc, root), "a & b <c>");
    }

    #[test]
    fn empty_document_is_error() {
        assert!(parse("").is_err());
    }

    #[test]
    fn undeclared_namespace_prefix_is_rejected() {
        // roxmltree is namespace-strict: a fragment using an undeclared prefix is
        // an error (the validators always navigate a single, fully-declared tree
        // rather than re-parsing namespace-incomplete subtrees).
        assert!(parse(r#"<saml:Assertion>x</saml:Assertion>"#).is_err());
    }

    #[test]
    fn pruned_descendant_skips_advice_subtree() {
        // The Advice subtree carries an inner Assertion with its own Issuer; the
        // pruned search must read only the outer Issuer.
        let xml = format!(
            r#"<saml:Assertion xmlns:saml="{NS_SAML}"><saml:Advice><saml:Assertion><saml:Issuer>INNER</saml:Issuer></saml:Assertion></saml:Advice><saml:Issuer>OUTER</saml:Issuer></saml:Assertion>"#
        );
        let doc = parse(&xml).unwrap();
        let root = doc.document_element();
        let issuer =
            find_descendant_pruned(&doc, root, (NS_SAML, "Issuer"), (NS_SAML, "Advice")).unwrap();
        assert_eq!(inner_text(&doc, issuer), "OUTER");
        // Only the outer Issuer is visible to the pruned descendant search.
        assert_eq!(
            descendants_by_tag_pruned(&doc, root, (NS_SAML, "Issuer"), (NS_SAML, "Advice")).len(),
            1
        );
        // Without pruning, both are found.
        assert_eq!(descendants_by_tag(&doc, root, NS_SAML, "Issuer").len(), 2);
    }
}
