use figma_mcp::figma::{FigmaUrlParser, FigmaUrlType};

#[test]
fn test_parse_file_url_basic() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/file/ABC123/my-design").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::File {
        file_id: "ABC123".to_string(),
        node_id: None,
    });
    assert_eq!(result.original_url, "https://www.figma.com/file/ABC123/my-design");
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
fn test_parse_file_url_with_additional_params() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/file/XYZ789/another-design?node-id=3%3A4&other=param").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::File {
        file_id: "XYZ789".to_string(),
        node_id: Some("3%3A4".to_string()),
    });
}

#[test]
fn test_parse_project_url_returns_unknown() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/files/project/123456").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::Unknown);
}

#[test]
fn test_parse_team_url_returns_unknown() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/files/team/789012").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::Unknown);
}

#[test]
fn test_parse_figma_url_without_www() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://figma.com/file/ABC123/my-design").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::File {
        file_id: "ABC123".to_string(),
        node_id: None,
    });
}

#[test]
fn test_parse_http_url() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("http://www.figma.com/file/ABC123/my-design").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::File {
        file_id: "ABC123".to_string(),
        node_id: None,
    });
}

#[test]
fn test_parse_invalid_url() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://example.com");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not a Figma URL"));
}

#[test]
fn test_parse_figma_url_unknown_format() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/unknown/path").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::Unknown);
}

#[test]
fn test_extract_file_id() {
    let parser = FigmaUrlParser::new();
    
    let file_id = parser.extract_file_id("https://www.figma.com/file/ABC123/my-design").unwrap();
    assert_eq!(file_id, "ABC123");
}

#[test]
fn test_extract_file_id_from_non_file_url() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.extract_file_id("https://www.figma.com/files/project/123456");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a file URL"));
}

#[test]
fn test_extract_file_id_from_project_url_fails() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.extract_file_id("https://www.figma.com/files/project/123456");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a file URL"));
}

#[test]
fn test_extract_file_id_from_team_url_fails() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.extract_file_id("https://www.figma.com/files/team/789012");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a file URL"));
}

#[test]
fn test_complex_file_url_with_path() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/file/ABC123/My-Design-Project/duplicate?node-id=1%3A2").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::File {
        file_id: "ABC123".to_string(),
        node_id: Some("1%3A2".to_string()),
    });
}

#[test]
fn test_url_with_alphanumeric_file_id() {
    let parser = FigmaUrlParser::new();
    
    let result = parser.parse("https://www.figma.com/file/Aa1Bb2Cc3/design").unwrap();
    assert_eq!(result.url_type, FigmaUrlType::File {
        file_id: "Aa1Bb2Cc3".to_string(),
        node_id: None,
    });
}