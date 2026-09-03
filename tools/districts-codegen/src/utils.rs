use eks_utils::slugify_teletex;
use eml_nl::documents::master_election_tree::MetRegion;

/// RegionNumber of the "NBSB" pseudo-municipality (Nationaal Brief Stembureau),
/// It only applies to TK and EP elections, which we don't generate districts for
/// yet, so it is filtered out of every municipality we generate here.
pub(crate) const NBSB_REGION_NUMBER: u16 = 9010;

/// Convert a region name to a valid PascalCase Rust identifier
pub(crate) fn to_ident(name: &str) -> String {
    slugify_teletex(name, false)
        .split('-')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut out: String = first.to_uppercase().collect();
                    out.push_str(chars.as_str());
                    out
                }
            }
        })
        .collect()
}

/// RegionNumber as u16, defaulting to 0 if absent
pub(crate) fn num(r: &MetRegion) -> u16 {
    r.key.number.unwrap_or(0)
}
