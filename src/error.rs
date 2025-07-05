use anyhow::Error as AnyhowError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Figma API error: {0}")]
    FigmaApi(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("MCP error: {0}")]
    Mcp(#[from] AnyhowError),
}