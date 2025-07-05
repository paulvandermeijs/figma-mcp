use reqwest::{Client, header::HeaderMap, header::HeaderValue};
use serde_json::Value;

use crate::{Error, Result};

const FIGMA_API_BASE: &str = "https://api.figma.com/v1";

#[derive(Debug, Clone)]
pub struct FigmaClient {
    client: Client,
    token: String,
}

impl FigmaClient {
    pub fn new(token: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Figma-Token", HeaderValue::from_str(&token)
            .map_err(|_| Error::Auth("Invalid token format".to_string()))?);
        
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::Network(e))?;

        Ok(Self { client, token })
    }

    pub async fn get_file(&self, file_id: &str, depth: Option<u32>) -> Result<Value> {
        let mut url = format!("{}/files/{}", FIGMA_API_BASE, file_id);
        if let Some(depth) = depth {
            url.push_str(&format!("?depth={}", depth));
        }
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::FigmaApi(format!("HTTP {}: {}", status, text)));
        }

        let json: Value = response.json().await?;
        
        if let Some(err) = json.get("err") {
            return Err(Error::FigmaApi(err.to_string()));
        }

        Ok(json)
    }

    pub async fn get_file_nodes(&self, file_id: &str, node_ids: &[String], depth: Option<u32>) -> Result<Value> {
        let ids = node_ids.join(",");
        let mut url = format!("{}/files/{}/nodes?ids={}", FIGMA_API_BASE, file_id, ids);
        if let Some(depth) = depth {
            url.push_str(&format!("&depth={}", depth));
        }
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::FigmaApi(format!("HTTP {}: {}", status, text)));
        }

        let json: Value = response.json().await?;
        
        if let Some(err) = json.get("err") {
            return Err(Error::FigmaApi(err.to_string()));
        }

        Ok(json)
    }



    pub async fn export_images(
        &self,
        file_id: &str,
        node_ids: &[String],
        format: &str,
        scale: Option<f64>,
    ) -> Result<Value> {
        let ids = node_ids.join(",");
        let mut url = format!(
            "{}/images/{}?ids={}&format={}",
            FIGMA_API_BASE, file_id, ids, format
        );
        
        if let Some(scale) = scale {
            url.push_str(&format!("&scale={}", scale));
        }

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::FigmaApi(format!("HTTP {}: {}", status, text)));
        }

        let json: Value = response.json().await?;
        
        if let Some(err) = json.get("err") {
            return Err(Error::FigmaApi(err.to_string()));
        }

        Ok(json)
    }

    pub async fn get_me(&self) -> Result<Value> {
        let url = format!("{}/me", FIGMA_API_BASE);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::FigmaApi(format!("HTTP {}: {}", status, text)));
        }

        let json: Value = response.json().await?;
        
        if let Some(err) = json.get("err") {
            return Err(Error::FigmaApi(err.to_string()));
        }

        Ok(json)
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = FigmaClient::new("test-token".to_string());
        assert!(client.is_ok());
    }

    #[tokio::test] 
    async fn test_invalid_token_format() {
        let client = FigmaClient::new("invalid\ntoken".to_string());
        assert!(client.is_err());
    }
}