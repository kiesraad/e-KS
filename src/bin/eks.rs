use eks::{AppError, AppState, logging, router, server};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // first arguments is the address to bind to
    let addr = std::env::args()
        .nth(1)
        .unwrap_or(std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:3000".to_string()));

    // Create a `TcpListener` using tokio.
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind to address {}: {}", addr, err);
            std::process::exit(1);
        }
    };

    // Run the application
    if let Err(err) = run(listener).await {
        eprintln!("Application error: {}", err);
        std::process::exit(1);
    }
}

async fn run(listener: TcpListener) -> Result<(), AppError> {
    // Initialize tracing subscriber (logging)
    logging::init();

    // Create application state
    let state = AppState::new()?;

    // Load fixtures, used for playwright tests
    #[cfg(feature = "fixtures")]
    {
        if std::env::var("LOAD_FIXTURES").is_ok() {
            // Run database migrations
            sqlx::migrate!()
                .run(state.pool())
                .await
                .expect("Failed to run migrations");

            // Load fixtures
            eks::fixtures::load(&state).await?;
        }
    }

    // Start the server
    server::serve(router::create(), state, listener).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, Uri},
    };
    use http_body_util::BodyExt;
    use hyper_util::rt::TokioExecutor;
    use tokio::net::TcpListener;

    async fn fetch(uri: Uri) -> (StatusCode, String) {
        let client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build_http();

        let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

        let resp = client.request(req).await.unwrap();
        let status = resp.status();

        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();

        let body_string = String::from_utf8(bytes.to_vec()).expect("utf-8 body");

        (status, body_string)
    }

    #[tokio::test]
    async fn serves_homepage_and_not_found() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            run(listener).await.unwrap();
        });

        let (status, body) = fetch(format!("http://{addr}/").parse().unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Kiesraad - Kandidaatstelling"));

        let (status, body) = fetch(format!("http://{addr}/missing").parse().unwrap()).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("Pagina niet gevonden"));

        server.abort();
    }
}
