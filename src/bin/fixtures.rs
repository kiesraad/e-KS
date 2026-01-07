use anyhow::Result;
use eks::{AppState, fixtures, logging};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file if present
    #[cfg(feature = "dev-features")]
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber (logging)
    logging::init();

    // Create application state
    let state = AppState::new()?;

    sqlx::migrate!().run(state.pool()).await?;

    fixtures::load(&state).await?;

    Ok(())
}
