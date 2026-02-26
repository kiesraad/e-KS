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
