use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::{Error, Result};

#[derive(Clone)]
pub struct ImageCache {
    entries: Arc<RwLock<HashMap<String, ImageEntry>>>,
}

#[derive(Clone, Debug)]
pub struct ImageEntry {
    pub file_key: String,
    pub node_id: String,
    pub format: String,
    pub scale: f64,
    pub figma_url: String,
    pub cached_data: Option<Vec<u8>>,
    pub export_time: SystemTime,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_export(
        &self,
        file_key: String,
        node_id: String,
        format: String,
        scale: f64,
        figma_url: String,
    ) -> Result<String> {
        let uri = Self::generate_uri(&file_key, &node_id, &format, scale);
        
        let entry = ImageEntry {
            file_key,
            node_id,
            format,
            scale,
            figma_url,
            cached_data: None,
            export_time: SystemTime::now(),
        };

        let mut entries = self.entries.write()
            .map_err(|_| Error::Internal("Failed to acquire lock".to_string()))?;
        entries.insert(uri.clone(), entry);

        Ok(uri)
    }

    pub fn list_all(&self) -> Result<Vec<(String, ImageEntry)>> {
        let entries = self.entries.read()
            .map_err(|_| Error::Internal("Failed to acquire lock".to_string()))?;
        
        Ok(entries.iter()
            .map(|(uri, entry)| (uri.clone(), entry.clone()))
            .collect())
    }

    pub fn get_entry(&self, uri: &str) -> Result<Option<ImageEntry>> {
        let entries = self.entries.read()
            .map_err(|_| Error::Internal("Failed to acquire lock".to_string()))?;
        
        Ok(entries.get(uri).cloned())
    }

    pub fn update_cached_data(&self, uri: &str, data: Vec<u8>) -> Result<()> {
        let mut entries = self.entries.write()
            .map_err(|_| Error::Internal("Failed to acquire lock".to_string()))?;
        
        if let Some(entry) = entries.get_mut(uri) {
            entry.cached_data = Some(data);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Resource not found: {}", uri)))
        }
    }

    pub fn is_expired(&self, entry: &ImageEntry) -> bool {
        if let Ok(elapsed) = entry.export_time.elapsed() {
            // Figma URLs typically expire after 1 hour
            elapsed.as_secs() > 3600
        } else {
            true
        }
    }

    pub fn get_mime_type(format: &str) -> &'static str {
        match format.to_lowercase().as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",
            "pdf" => "application/pdf",
            _ => "application/octet-stream",
        }
    }

    fn generate_uri(file_key: &str, node_id: &str, format: &str, scale: f64) -> String {
        if scale != 1.0 {
            format!("figma://file/{}/node/{}@{}x.{}", file_key, node_id, scale as u32, format)
        } else {
            format!("figma://file/{}/node/{}.{}", file_key, node_id, format)
        }
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}