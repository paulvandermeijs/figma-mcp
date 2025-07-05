use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FigmaUrlType {
    File { file_id: String, node_id: Option<String> },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigmaUrlInfo {
    pub url_type: FigmaUrlType,
    pub original_url: String,
}

#[derive(Debug, Clone)]
pub struct FigmaUrlParser {
    file_regex: Regex,
}

impl FigmaUrlParser {
    pub fn new() -> Self {
        Self {
            file_regex: Regex::new(r"^https?://(?:www\.)?figma\.com/(?:file|design)/([A-Za-z0-9]+)(?:/[^?]*)?(?:\?.*node-id=([^&]+))?")
                .expect("Invalid file regex"),
        }
    }

    pub fn parse(&self, url_str: &str) -> Result<FigmaUrlInfo> {
        let url = Url::parse(url_str)?;
        
        if !self.is_figma_url(&url) {
            return Err(Error::InvalidUrl(format!("Not a Figma URL: {}", url_str)));
        }

        let url_type = if let Some(captures) = self.file_regex.captures(url_str) {
            let file_id = captures.get(1).unwrap().as_str().to_string();
            let node_id = captures.get(2).map(|m| m.as_str().to_string());
            FigmaUrlType::File { file_id, node_id }
        } else {
            FigmaUrlType::Unknown
        };

        Ok(FigmaUrlInfo {
            url_type,
            original_url: url_str.to_string(),
        })
    }

    pub fn extract_file_id(&self, url_str: &str) -> Result<String> {
        match self.parse(url_str)? {
            FigmaUrlInfo { url_type: FigmaUrlType::File { file_id, .. }, .. } => Ok(file_id),
            _ => Err(Error::InvalidUrl(format!("URL is not a file URL: {}", url_str))),
        }
    }


    fn is_figma_url(&self, url: &Url) -> bool {
        matches!(url.host_str(), Some("figma.com") | Some("www.figma.com"))
    }
}

impl Default for FigmaUrlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_url() {
        let parser = FigmaUrlParser::new();
        
        let result = parser.parse("https://www.figma.com/file/ABC123/my-design").unwrap();
        assert_eq!(result.url_type, FigmaUrlType::File {
            file_id: "ABC123".to_string(),
            node_id: None,
        });
    }

    #[test]
    fn test_parse_file_url_with_node() {
        let parser = FigmaUrlParser::new();
        
        let result = parser.parse("https://www.figma.com/file/ABC123/my-design?node-id=1%3A2").unwrap();
        assert_eq!(result.url_type, FigmaUrlType::File {
            file_id: "ABC123".to_string(),
            node_id: Some("1%3A2".to_string()),
        });
    }


    #[test]
    fn test_parse_invalid_url() {
        let parser = FigmaUrlParser::new();
        
        let result = parser.parse("https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_non_file_figma_url() {
        let parser = FigmaUrlParser::new();
        
        let result = parser.parse("https://www.figma.com/files/project/123456").unwrap();
        assert_eq!(result.url_type, FigmaUrlType::Unknown);
    }

    #[test]
    fn test_extract_file_id() {
        let parser = FigmaUrlParser::new();
        
        let file_id = parser.extract_file_id("https://www.figma.com/file/ABC123/my-design").unwrap();
        assert_eq!(file_id, "ABC123");
    }

    #[test]
    fn test_parse_design_url() {
        let parser = FigmaUrlParser::new();
        
        let result = parser.parse("https://www.figma.com/design/ABC123/my-design").unwrap();
        assert_eq!(result.url_type, FigmaUrlType::File {
            file_id: "ABC123".to_string(),
            node_id: None,
        });
    }

    #[test]
    fn test_parse_design_url_with_node() {
        let parser = FigmaUrlParser::new();
        
        let result = parser.parse("https://www.figma.com/design/ABC123/my-design?node-id=201-95620").unwrap();
        assert_eq!(result.url_type, FigmaUrlType::File {
            file_id: "ABC123".to_string(),
            node_id: Some("201-95620".to_string()),
        });
    }

    #[test]
    fn test_extract_file_id_from_design_url() {
        let parser = FigmaUrlParser::new();
        
        let file_id = parser.extract_file_id("https://www.figma.com/design/mDRPCttt3pWEmznGjW8JPg/Visual-design-RET").unwrap();
        assert_eq!(file_id, "mDRPCttt3pWEmznGjW8JPg");
    }
}