/// Serialize a filter struct to a query-string fragment with leading `&`,
/// or an empty string when no filters are set. Appended to pagination links.
pub fn filter_query_suffix(filter: &impl serde::Serialize) -> String {
    match serde_urlencoded::to_string(filter) {
        Ok(query) if !query.is_empty() => format!("&{query}"),
        _ => String::new(),
    }
}
