use figma_mcp::figma::FigmaClient;

#[tokio::test]
async fn test_client_creation() {
    let client = FigmaClient::new("test-token".to_string());
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_invalid_token_format() {
    // Test with newline in token (should fail)
    let client = FigmaClient::new("invalid\ntoken".to_string());
    assert!(client.is_err());
}

// Note: For now, these tests just verify the client structure
// In a full implementation, we would:
// 1. Make the client configurable to use different base URLs
// 2. Set up proper mock server integration
// 3. Test actual API calls with mocked responses

#[test]
fn test_client_token_storage() {
    let token = "test-token-123".to_string();
    let client = FigmaClient::new(token.clone()).unwrap();
    assert_eq!(client.get_token(), &token);
}

#[test]
fn test_client_debug_and_clone() {
    let client = FigmaClient::new("test-token".to_string()).unwrap();
    let cloned = client.clone();
    assert_eq!(client.get_token(), cloned.get_token());
    
    // Ensure Debug trait works
    let debug_output = format!("{:?}", client);
    assert!(debug_output.contains("FigmaClient"));
}