use eks::{AppError, AppState, Config, logging, router, run_db_prober, server};
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

    // Load and validate configuration before binding so misconfiguration fails fast.
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("Invalid configuration: {err}");
            std::process::exit(1);
        }
    };

    // Create a `TcpListener` using tokio.
    let listener = match TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!("Failed to bind to address {address}: {err}");
            std::process::exit(1);
        }
    };

    // Run the application
    if let Err(err) = run(listener, config).await {
        tracing::error!("Application error: {}", err);
        std::process::exit(1);
    }
}

/// Runs the application with the given TCP listener and resolved configuration.
/// Initializes application state, builds the router, and starts the server.
async fn run(listener: TcpListener, config: Config) -> Result<(), AppError> {
    // Create application state
    let state = AppState::new_with_config(config).await?;

    // Stores are loaded per political group on demand via StoreRegistry.

    // Keep the database-health gate current and self-heal (re-run migrations)
    // when the database recovers, without blocking startup or requiring a
    // restart. The application starts even if the database is currently down.
    tokio::spawn(run_db_prober(
        state.store_registry.persistence().clone(),
        state.db_health.clone(),
    ));

    // Start the server
    let router = router::create(state.clone()).with_state(state.clone());
    server::serve(router, listener, state.config).await?;

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
        let config = Config::from_env().expect("config");
        let server = tokio::spawn(async move {
            run(listener, config).await.unwrap();
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
    async fn start_serves_the_application() {
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

        // SAML /login can't be exercised end-to-end without an IdP; use the
        // dev-features shortcut to mint a session. dev_login attaches a default
        // election (EK27) and redirects to "/", so verify the index page renders.
        let cookie = dev_login(&base).await;

        let (status, body) = fetch_with_cookie(&format!("{base}/"), &cookie).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Kiesraad - Kandidaatstelling"));

        server.abort();
    }
}
