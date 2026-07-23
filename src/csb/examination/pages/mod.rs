use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

use super::paths::*;

mod all_restorations;
mod candidate;
mod candidate_list;
mod correction;
mod general_information;
mod i4;
mod omission;
mod overview;
mod paper_corrections;
mod political_group;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(overview::overview)
        .typed_get(i4::gen_i4)
        .typed_get(political_group::overview)
        .typed_post(political_group::toggle_examination_finish)
        .typed_get(general_information::overview)
        .typed_post(paper_corrections::start_paper_corrections)
        .typed_post(paper_corrections::stop_paper_corrections)
        .typed_get(candidate_list::overview)
        .typed_get(candidate::overview)
        .typed_get(omission::add_omission)
        .typed_post(omission::add_omission_submit)
        .typed_get(omission::overview)
        .typed_post(omission::delete_omission)
        .typed_get(all_restorations::all_restorations)
        .typed_get(correction::display_name_correction)
        .typed_post(correction::display_name_correction_submit)
        .typed_get(correction::person_correction)
        .typed_post(correction::person_correction_submit)
}
