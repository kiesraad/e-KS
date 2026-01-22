use axum_extra::routing::TypedPath;

mod create;

#[derive(TypedPath)]
#[typed_path("/political_group/new", rejection(crate::AppError))]
pub struct PoliticalGroupNewPath;
