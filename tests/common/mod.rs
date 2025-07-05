use std::fs;
use std::path::Path;

pub fn load_fixture(fixture_path: &str) -> String {
    let path = Path::new("tests/fixtures").join(fixture_path);
    fs::read_to_string(path).expect(&format!("Failed to read fixture: {}", fixture_path))
}

pub fn load_json_fixture<T>(fixture_path: &str) -> T 
where
    T: serde::de::DeserializeOwned,
{
    let content = load_fixture(fixture_path);
    serde_json::from_str(&content).expect(&format!("Failed to parse JSON fixture: {}", fixture_path))
}

#[allow(dead_code)]
pub fn mock_figma_token() -> String {
    "test-figma-token-123456".to_string()
}