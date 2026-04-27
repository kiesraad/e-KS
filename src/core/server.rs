//! Server startup and graceful shutdown for the Axum application.
//! Supports plain HTTP via `axum::serve`, and (with the `tls` feature) HTTPS
//! via `axum-server` with rustls. Called from binaries to run the router
//! with AppState.

use axum::Router;
use tokio::{net::TcpListener, signal};

use crate::AppError;

#[cfg(feature = "tls")]
pub use tls::TlsConfig;

pub async fn serve(router: Router, listener: TcpListener) -> Result<(), AppError> {
    let addr = listener.local_addr().map_err(AppError::ServerError)?;

    tracing::info!("Starting server on http://{addr}");

    // Run the server with graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(AppError::ServerError)?;

    Ok(())
}

#[cfg(feature = "tls")]
pub async fn serve_tls(
    router: Router,
    listener: TcpListener,
    tls: TlsConfig,
) -> Result<(), AppError> {
    tls::serve(router, listener, tls).await
}

#[cfg(feature = "tls")]
mod tls {
    #[cfg(not(debug_assertions))]
    use std::time::Duration;
    use std::{net::SocketAddr, path::PathBuf};

    use axum::Router;
    use axum_server::{Handle, tls_rustls::RustlsConfig};
    use tokio::net::TcpListener;

    use crate::AppError;

    /// TLS configuration for serving HTTPS via rustls.
    #[derive(Debug, Clone)]
    pub struct TlsConfig {
        pub cert_path: PathBuf,
        pub key_path: PathBuf,
    }

    impl TlsConfig {
        /// Read TLS configuration from `TLS_CERT_PATH` and `TLS_KEY_PATH`.
        /// Both must be set together; if neither is set, returns `Ok(None)`.
        pub fn from_env() -> Result<Option<Self>, AppError> {
            let cert = std::env::var("TLS_CERT_PATH").ok();
            let key = std::env::var("TLS_KEY_PATH").ok();
            match (cert, key) {
                (Some(cert), Some(key)) => Ok(Some(Self {
                    cert_path: PathBuf::from(cert),
                    key_path: PathBuf::from(key),
                })),
                (None, None) => Ok(None),
                _ => Err(AppError::ConfigLoadError(
                    "TLS_CERT_PATH and TLS_KEY_PATH must both be set, or both unset".to_string(),
                )),
            }
        }
    }

    pub async fn serve(
        router: Router,
        listener: TcpListener,
        tls: TlsConfig,
    ) -> Result<(), AppError> {
        let addr = listener.local_addr().map_err(AppError::ServerError)?;

        let config = RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path)
            .await
            .map_err(AppError::ServerError)?;

        tracing::info!("Starting server on https://{addr}");

        let std_listener = listener.into_std().map_err(AppError::ServerError)?;

        let handle: Handle<SocketAddr> = Handle::new();
        tokio::spawn(graceful_shutdown_handle(handle.clone()));

        axum_server::from_tcp_rustls(std_listener, config)
            .map_err(AppError::ServerError)?
            .handle(handle)
            .serve(router.into_make_service())
            .await
            .map_err(AppError::ServerError)?;

        Ok(())
    }

    async fn graceful_shutdown_handle(handle: Handle<SocketAddr>) {
        super::wait_for_signal().await;
        #[cfg(not(debug_assertions))]
        {
            tracing::info!("Received shutdown signal, gracefully shutting down.");
            handle.graceful_shutdown(Some(Duration::from_secs(10)));
        }
        #[cfg(debug_assertions)]
        {
            let _ = handle;
            tracing::info!("Received shutdown signal, no graceful shutdown in development mode.");
            std::process::exit(0);
        }
    }
}

async fn wait_for_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(not(debug_assertions))]
async fn shutdown_signal() {
    wait_for_signal().await;
    tracing::info!("Received shutdown signal, gracefully shutting down.");
}

#[cfg(debug_assertions)]
async fn shutdown_signal() {
    wait_for_signal().await;
    tracing::info!("Received shutdown signal, no graceful shutdown in development mode.");
    std::process::exit(0);
}
