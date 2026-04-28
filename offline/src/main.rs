// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

const WINDOW_WIDTH: f64 = 1500.0;
const WINDOW_HEIGHT: f64 = 1000.0;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Bind to an ephemeral local port (avoid hardcoding / conflicts)
            let std_listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
            std_listener.set_nonblocking(true)?;
            let addr = std_listener.local_addr()?;

            tauri::async_runtime::spawn(async move {
                // Initialize tracing subscriber (logging)
                eks::logging::init();

                // Convert to Tokio listener and spawn Axum
                let tokio_listener = tokio::net::TcpListener::from_std(std_listener)
                    .map_err(eks::AppError::ServerError)?;

                // Create application state. PDFs are rendered in-process via
                // the embedded Typst library (feature `embed-typst`).
                let state = eks::AppState::new().await?;

                // Start the server
                let router = eks::router::create(state.clone()).with_state(state);
                eks::server::serve(router, tokio_listener).await?;

                Ok::<(), eks::AppError>(())
            });

            // Create the main window pointing at http://127.0.0.1:<port>/
            let url = url::Url::parse(&format!("http://{addr}/"))?;
            tauri::WebviewWindowBuilder::new(app, "eks", tauri::WebviewUrl::External(url))
                .title("e-KS")
                .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .resizable(true)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
