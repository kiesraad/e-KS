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

/// Starts the server on the given address. If the "embed-typst" feature is
/// enabled, PDFs are rendered in-process using the embedded typst-webservice
/// library; otherwise an external typst-webservice is contacted over HTTP.
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

    // Run the application
    if let Err(err) = run(listener).await {
        tracing::error!("Application error: {}", err);
        std::process::exit(1);
    }
}

/// Runs the application with the given TCP listener. Initializes logging, application state, loads data, and starts the server.
async fn run(listener: TcpListener) -> Result<(), AppError> {
    // Create application state
    let state = AppState::new().await?;

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
    use reqwest::Client;
    use std::net::TcpListener as StdTcpListener;
    use tokio::{
        net::TcpListener,
        time::{Duration, sleep},
    };

    async fn fetch_with_cookie(url: &str, cookie: &str) -> (StatusCode, String) {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let resp = client
            .get(url)
            .header("Cookie", cookie)
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body = resp.text().await.expect("body text");
        (status, body)
    }

    async fn dev_login(base: &str) -> String {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let url = format!("{base}/dev/login?bsn=999999990&fixtures=false");
        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        resp.headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .expect("session cookie")
            .to_string()
    }

    #[cfg_attr(not(feature = "net-tests"), ignore = "requires network")]
    #[tokio::test]
    async fn serves_homepage_and_not_found() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            run(listener).await.unwrap();
        });

        let base = format!("http://{addr}");
        let cookie = dev_login(&base).await;

        let (status, body) = fetch_with_cookie(&format!("{base}/"), &cookie).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Kiesraad - Kandidaatstelling"));

        let (status, body) = fetch_with_cookie(&format!("{base}/missing"), &cookie).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("Pagina niet gevonden"));

        server.abort();
    }

    #[cfg_attr(not(feature = "net-tests"), ignore = "requires network")]
    #[tokio::test]
    async fn start_binds_and_serves_login_flow() {
        let port = StdTcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();

        let address = format!("127.0.0.1:{port}");
        let server = tokio::spawn(async move {
            start(address).await;
        });

        let base = format!("http://127.0.0.1:{port}");
        let no_redirect = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        let mut ready = false;
        for _ in 0..20 {
            match no_redirect.get(format!("{base}/login")).send().await {
                Ok(_) => {
                    ready = true;
                    break;
                }
                Err(_) => {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }
        assert!(ready, "server never became ready");

        // /login creates a new session and redirects to /select-election
        // (no existing election for a fresh user)
        let resp = no_redirect
            .get(format!("{base}/login"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        assert_eq!(location, "/select-election");
        let cookie = resp
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .expect("session cookie")
            .to_string();

        // Follow the redirect to /select-election with the session cookie
        let (status, body) = fetch_with_cookie(&format!("{base}{location}"), &cookie).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("select-election") || body.contains("Verkiezing"));

        server.abort();
    }
}
