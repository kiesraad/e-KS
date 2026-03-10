use eks::{AppError, AppState, logging, router, server};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // first arguments is the address to bind to
    let address = std::env::args()
        .nth(1)
        .unwrap_or(std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:3000".to_string()));

    start(address).await;
}

/// Starts the server on the given address. If the "embed-typst" feature is enabled, also starts the embedded typst webservice.
async fn start(address: String) {
    // Initialize tracing subscriber (logging)
    logging::init();

    // Create a `TcpListener` using tokio.
    let listener = match TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!("Failed to bind to address {address}: {err}");
            std::process::exit(1);
        }
    };

    // Start embedded typst webservice if the feature is enabled
    #[cfg(feature = "embed-typst")]
    let typst_url = match eks::utils::embed_typst::start().await {
        Ok(url) => Some(url),
        Err(err) => {
            tracing::error!("Failed to start typst webservice: {err}");
            std::process::exit(1);
        }
    };

    #[cfg(not(feature = "embed-typst"))]
    let typst_url = None;

    // Run the application
    if let Err(err) = run(listener, typst_url).await {
        tracing::error!("Application error: {}", err);
        std::process::exit(1);
    }
}

/// Runs the application with the given TCP listener and optional typst URL. Initializes logging, application state, loads data, and starts the server.
async fn run(listener: TcpListener, typst_url: Option<String>) -> Result<(), AppError> {
    // Create application state
    let state = AppState::new(typst_url).await?;

    // Stores are loaded per political group on demand via StoreRegistry.

    // Start the server
    let router = router::create(state.clone()).with_state(state.clone());
    server::serve(router, listener).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use reqwest::{Client, Error as ReqwestError};
    use std::net::TcpListener as StdTcpListener;
    use tokio::{
        net::TcpListener,
        time::{Duration, sleep},
    };

    async fn fetch(url: &str) -> (StatusCode, String) {
        let client = Client::new();
        let resp = client.get(url).send().await.unwrap();
        let status = resp.status();
        let body = resp.text().await.expect("body text");
        (status, body)
    }

    async fn try_fetch(url: &str) -> Result<(StatusCode, String), ReqwestError> {
        let client = Client::new();
        let resp = client.get(url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        Ok((status, body))
    }

    #[cfg_attr(not(feature = "net-tests"), ignore = "requires network")]
    #[tokio::test]
    async fn serves_homepage_and_not_found() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            run(listener, None).await.unwrap();
        });

        let (status, body) = fetch(&format!("http://{addr}/")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Kiesraad - Kandidaatstelling"));

        let (status, body) = fetch(&format!("http://{addr}/missing")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("Pagina niet gevonden"));

        server.abort();
    }

    #[cfg_attr(not(feature = "net-tests"), ignore = "requires network")]
    #[tokio::test]
    async fn start_binds_and_serves_homepage() {
        let port = StdTcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();

        let address = format!("127.0.0.1:{port}");
        let server = tokio::spawn(async move {
            start(address).await;
        });

        let url = format!("http://127.0.0.1:{port}/");

        let mut ready = None;
        for _ in 0..20 {
            match try_fetch(&url).await {
                Ok(result) => {
                    ready = Some(result);
                    break;
                }
                Err(_) => {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }

        let (status, body) = ready.expect("server never became ready");
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Kiesraad - Kandidaatstelling"));

        server.abort();
    }
}
