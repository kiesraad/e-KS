/// Event type category grouping specific event keys, used by the audit-log
/// filter dropdowns (app and CSB) to render `<optgroup>`s with fine-grained
/// `<option>`s.
pub struct EventTypeCategory {
    pub key: &'static str,
    pub event_types: &'static [&'static str],
}
