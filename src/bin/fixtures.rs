use eks::{AppState, fixtures, logging};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber (logging)
    logging::init();

    // Create application state
    let state = AppState::new()?;

    sqlx::migrate!().run(state.pool()).await?;

    fixtures::load(&state).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error loading fixtures: {:?}", err);
        std::process::exit(1);
    }
}
