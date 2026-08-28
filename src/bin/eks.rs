use eks::{
    AppError, AppState, Config, logging, router, run_db_prober, run_session_sweeper, server,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // first arguments is the address to bind to
    let address = std::env::args()
        .nth(1)
        .unwrap_or(std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0:3000".to_string()));

    start(address).await;
}

/// Unwraps a startup step, logging the error and exiting on failure.
fn or_exit<T>(result: Result<T, impl std::fmt::Display>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => {
            tracing::error!("{context}: {err}");
            std::process::exit(1);
        }
    }
}

/// Starts the server on the given address.
async fn start(address: String) {
    // Initialize tracing subscriber (logging)
    logging::init();

    // Load and validate configuration before binding so misconfiguration fails fast.
    let config = or_exit(Config::from_env(), "Invalid configuration");

    let listener = or_exit(
        TcpListener::bind(&address).await,
        &format!("Failed to bind to address {address}"),
    );

    or_exit(run(listener, config).await, "Application error");
}

/// Runs the application with the given TCP listener and resolved configuration.
/// Initializes application state, builds the router, and starts the server.
async fn run(listener: TcpListener, config: Config) -> Result<(), AppError> {
    // Create application state
    let state = AppState::new_with_config(config).await?;

    // Stores are loaded per political group on demand via StoreRegistry.

    // Keep the database-health gate current without blocking startup or
    // requiring a restart. The prober runs migrations once, the first time the
    // database is reachable (so a database that is down at boot is migrated on
    // recovery), then only verifies the schema. The application starts even if
    // the database is currently down.
    tokio::spawn(run_db_prober(
        state.store_registry.persistence().clone(),
        state.db_health.clone(),
    ));

    // Periodically evict expired sessions (Postgres backend accumulates rows
    // otherwise).
    tokio::spawn(run_session_sweeper(state.sessions.clone()));

    // Start the server
    let router = router::create(state.clone()).with_state(state.clone());

    // The renewer hot-reloads renewed certificates through a clone of the
    // TLS config the server listens with.
    #[cfg(feature = "acme")]
    if let (Some(tls), Some(acme)) = (state.config.tls.as_ref(), state.config.acme.as_ref()) {
        // Fail fast on malformed account credentials.
        let _ = eks::parse_acme_account_credentials(acme)?;
        eks::bootstrap_certificate(acme, tls).await?;
        let rustls_config = server::build_rustls_config(tls).await?;
        tokio::spawn(eks::run_acme_renewer(
            acme,
            tls,
            rustls_config.clone(),
            state.acme_store.clone(),
        ));
        return server::serve_tls(router, listener, rustls_config).await;
    }

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
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|v| v.split(';').next())
            .find(|pair| pair.starts_with(eks::SESSION_COOKIE_NAME))
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
