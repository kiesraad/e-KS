use typst_webservice::{PdfContext, start_server};

const TYPST_FILES: &[(&str, &[u8])] = &[
    (
        "DMSans-Bold.ttf",
        include_bytes!("../../models/fonts/DMSans-Bold.ttf"),
    ),
    (
        "DMSans-BoldItalic.ttf",
        include_bytes!("../../models/fonts/DMSans-BoldItalic.ttf"),
    ),
    (
        "DMSans-Regular.ttf",
        include_bytes!("../../models/fonts/DMSans-Regular.ttf"),
    ),
    (
        "DMSans-Italic.ttf",
        include_bytes!("../../models/fonts/DMSans-Italic.ttf"),
    ),
    (
        "GeistMono-Regular.otf",
        include_bytes!("../../models/fonts/GeistMono-Regular.otf"),
    ),
    ("layout.typ", include_bytes!("../../models/layout.typ")),
    (
        "model-h-1.typ",
        include_bytes!("../../models/model-h-1.typ"),
    ),
];

pub async fn start() -> Result<String, std::io::Error> {
    // bind to random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tracing::info!("Typst webservice listening on {address}");

    // Start the typst webservice in the background
    tokio::spawn(async move {
        let context = PdfContext::from_assets(TYPST_FILES).unwrap();
        start_server(listener, context).await.unwrap();
    });

    Ok(format!("http://{address}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::SocketAddr, str::FromStr};
    use tokio::{
        net::TcpStream,
        time::{Duration, sleep, timeout},
    };

    async fn wait_for_server(addr: SocketAddr) -> bool {
        for _ in 0..10 {
            if TcpStream::connect(addr).await.is_ok() {
                return true;
            }
            sleep(Duration::from_millis(25)).await;
        }
        false
    }

    #[tokio::test]
    async fn start_returns_local_url_and_accepts_connections() {
        let url = start().await.expect("start typst server");
        assert!(url.starts_with("http://127.0.0.1:"));

        let addr = url
            .strip_prefix("http://")
            .and_then(|value| SocketAddr::from_str(value).ok())
            .expect("valid socket address");

        let ready = timeout(Duration::from_secs(2), wait_for_server(addr))
            .await
            .unwrap_or(false);

        assert!(ready, "typst server did not accept connections");
    }
}
