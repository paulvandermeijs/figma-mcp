use figma_mcp::{server::FigmaServer, Result};
use std::env;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // Get Figma token from environment
    let figma_token = env::var("FIGMA_TOKEN")
        .map_err(|_| figma_mcp::Error::Auth(
            "FIGMA_TOKEN environment variable not set. Get your token from: https://www.figma.com/developers/api#access-tokens".to_string()
        ))?;

    // Create and start the server
    let server = FigmaServer::new(figma_token)?;
    server.run_stdio().await?;

    Ok(())
}
