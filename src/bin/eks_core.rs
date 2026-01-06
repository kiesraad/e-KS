use askama::Template;
use axum::{Router, extract::Query, response::Html, routing::get};
use serde::Deserialize;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router()).await.unwrap();
}

fn router() -> Router {
    Router::new().route("/", get(index))
}

#[derive(Deserialize)]
struct Params {
    name: Option<String>,
}

async fn index(Query(Params { name }): Query<Params>) -> Html<String> {
    let name = name.as_deref().unwrap_or("world");

    Html(Index { name }.render().unwrap())
}

#[derive(askama::Template)]
#[template(path = "index.j2")]
struct Index<'a> {
    name: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::router;
    use axum::{body::Body, extract::Request, http::StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn hello_world() {
        let request = Request::builder()
            .method("GET")
            .uri("/?a=true&b=true")
            .body(Body::empty())
            .unwrap();
        let response = router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK)
    }
}
