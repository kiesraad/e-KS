use crate::{pg::request_extractor, political_groups::PoliticalGroup};

request_extractor!(PoliticalGroup, |store, parts, state| {
    Ok(store.get_political_group())
});
