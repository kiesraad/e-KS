//! Typed paths for the CSB index route.

use axum_extra::routing::TypedPath;

use crate::AppError;

#[derive(TypedPath)]
#[typed_path("/csb", rejection(AppError))]
pub struct CsbIndexPath;

#[derive(TypedPath)]
#[typed_path("/csb/eml110a.eml.xml", rejection(AppError))]
pub struct CsbElectionDefinitionDownloadPath;
