use eks::{AppError, AppState, logging, router, server};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load environment variables from a .env file if present
    #[cfg(feature = "dotenv")]
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber (logging)
    logging::init();

    // Create application state
    let state = AppState::new()?;

    // Start the server
    server::serve(router::create(), state).await?;

    Ok(())
}
