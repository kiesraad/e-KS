use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/robots.txt")]
pub struct RobotsTxt {}

/// Emit a robots.txt that disallows all crawling
pub(super) async fn robots_txt(_: RobotsTxt) -> &'static str {
    "User-agent: *\nDisallow: /\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::response_body_string;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn robots_disallows_everything() {
        let text = robots_txt(RobotsTxt {}).await;
        let body = response_body_string(text.into_response()).await;
        assert_eq!(body, "User-agent: *\nDisallow: /\n");
    }
}
