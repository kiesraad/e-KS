//! Server startup and graceful shutdown for the Axum application.
//! Called from binaries to run the router with AppState.

use axum::Router;
use tokio::{net::TcpListener, signal};

use crate::{AppError, AppState};

pub async fn serve(
    router: Router<AppState>,
    state: AppState,
    listener: TcpListener,
) -> Result<(), AppError> {
    let app = router.with_state(state);
    let addr = listener.local_addr().map_err(AppError::ServerError)?;

    tracing::info!("Starting server on {addr}");

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(AppError::ServerError)?;

    Ok(())
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
