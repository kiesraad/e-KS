use eks::{AppError, AppState, logging, router, server};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing subscriber (logging)
    logging::init();

    // Create application state
    let state = AppState::new()?;

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
    server::serve(router::create(), state).await?;

    Ok(())
}
