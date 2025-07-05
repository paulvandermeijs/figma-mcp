# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Running
```bash
# Development build and check
cargo check
cargo build

# Release build (for distribution)
cargo build --release

# Run the MCP server (requires FIGMA_TOKEN environment variable)
FIGMA_TOKEN="your_token" cargo run

# Run with debug logging
RUST_LOG=debug FIGMA_TOKEN="your_token" cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test module
cargo test url_parsing
cargo test api_client

# Run tests with output and logging
cargo test -- --nocapture
RUST_LOG=debug cargo test
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

## High-Level Architecture

This is a **Model Context Protocol (MCP) server** that provides AI assistants with tools to access Figma files via Figma's API. The server focuses on file-based operations using a two-step workflow: first parse URLs to extract file keys, then use those keys with file operation tools. Exported images are automatically available as MCP resources with base64-encoded content.

### Core Architecture Pattern

The codebase follows a **layered architecture** with clear separation between:

1. **Protocol Layer** (`src/server.rs`): MCP server implementation using the `rmcp` crate
2. **Business Logic Layer** (`src/figma/`): Figma-specific functionality  
3. **Infrastructure Layer**: HTTP client, error handling, URL parsing

### Key Components

**URL Parser (`src/figma/url_parser.rs`)**
- Central component that extracts file IDs from various Figma URL formats
- Uses regex patterns to handle different URL variations (file and design URLs)
- Returns structured `FigmaUrlInfo` with parsed components
- Critical for the URL-first approach due to Figma API limitations

**API Client (`src/figma/client.rs`)**
- Returns `serde_json::Value` instead of typed structs for flexibility
- Handles Figma authentication via personal access tokens
- Implements file-focused Figma API endpoints (files, nodes, image export, user info)
- Comprehensive error handling for API failures and rate limiting (60 req/min)

**Image Cache (`src/figma/image_cache.rs`)**
- Manages exported images as MCP resources
- Thread-safe storage using `Arc<RwLock<HashMap>>`
- Tracks Figma URLs, export metadata, and cached image data
- Handles URL expiration and image data caching

**MCP Server (`src/server.rs`)**
- Implements 6 MCP tools using `#[tool]` attribute macros focused on file operations
- Uses typed parameter structs with `#[derive(JsonSchema)]` for proper MCP Inspector integration
- All tools follow pattern: `Parameters<StructName>` for parameter binding
- Returns JSON strings via `CallToolResult::success()`
- Implements resource handlers for listing and reading exported images
- Downloads and base64-encodes images on demand

**Error Handling (`src/error.rs`)**
- Custom error enum covering all failure modes (API, network, auth, JSON, URL parsing)
- User-friendly error messages throughout the application
- Proper error propagation using `?` operator and `thiserror`

### MCP Tools Architecture

All tools focus on file-based operations with a clear separation between URL parsing and file operations:

**URL Parsing Tool**:
- `parse_figma_url` - Parse URLs to extract file keys and node information

**File Operation Tools** (require file key from `parse_figma_url`):
- `get_file` - Complete file data extraction using file key with depth control (default: 1)
- `get_file_nodes` - Specific node data using file key with depth control (default: 1)
- `export_images` - Image export using file key

**Utility Tools**:
- `get_me` - Authentication testing
- `help` - Usage instructions

### Parameter Schema System

The server uses a sophisticated parameter schema system:
- Each tool has a dedicated parameter struct (e.g., `ParseUrlRequest`, `GetFileRequest`)
- Structs derive `JsonSchema` for MCP Inspector integration
- Field-level documentation via `#[schemars(description = "...")]`
- Optional parameters use `Option<T>`
- Parameters bound via `Parameters<StructName>` pattern in tool signatures

### Data Flow Pattern

1. **Input**: User provides Figma URL through MCP client
2. **Parse**: `parse_figma_url` tool extracts file key and node information
3. **File Operations**: User provides file key to file operation tools (`get_file`, `get_file_nodes`, `export_images`) with optional depth control
4. **Fetch**: API client makes authenticated requests to Figma using file key and depth parameter
5. **Return**: Raw JSON data returned as flexible `serde_json::Value` (limited by depth to manage response size)
6. **Navigation**: For large files, use recursive calls with specific node IDs to explore deeper
7. **Output**: JSON serialized and returned via MCP protocol

This design enforces a **clear separation of concerns** and **response size management** - URL parsing is separate from file operations, and depth control prevents token limit issues while enabling incremental exploration of large Figma files.

### Depth Parameter and Response Size Management

The server implements Figma's depth parameter to control response size and prevent LLM token limit issues:

**Depth Behavior:**
- **For `get_file`**: Depth controls document tree traversal from root (1=pages only, 2=pages+objects, etc.)
- **For `get_file_nodes`**: Depth controls traversal from specified nodes (1=direct children, 2=children+grandchildren, etc.)
- **Default depth**: 1 (shallow traversal to minimize response size)

**Recursive Navigation Strategy:**
1. Start with `get_file` at depth=1 to understand file structure
2. Use `get_file_nodes` with specific page/frame IDs for targeted exploration
3. Incrementally increase depth or target specific nodes based on analysis needs
4. This approach allows LLM clients to explore large files without hitting token limits

### Testing Strategy

- **Unit tests** (`tests/unit/`): URL parsing and client logic with mocked responses
- **Test fixtures** (`tests/fixtures/`): Sample Figma API responses for realistic testing  
- **Integration approach**: Focus on individual component testing rather than full end-to-end
- **Error testing**: Comprehensive coverage of failure scenarios

### Critical Implementation Details

**rmcp Integration**: Uses the official Rust MCP SDK with `#[tool_router]` and `#[tool_handler]` macros for automatic tool discovery and routing.

**Authentication**: Requires `FIGMA_TOKEN` environment variable. Token passed in `X-Figma-Token` header for all API requests.

**Rate Limiting**: Figma enforces 60 requests/minute. Client provides clear error messages but no retry logic to avoid complexity.

**URL Flexibility**: Supports multiple Figma file URL formats including legacy `/file/` and newer `/design/` paths, with and without node parameters.

## Code Style and Patterns

### Reducing Indentation and Nested Expressions

The codebase prioritizes **readability through reduced nesting** and **clear linear flow**:

**Use match with early return** to flatten nested logic:
```rust
// Instead of nested match expressions
let file_id = match self.url_parser.extract_file_id(&url) {
    Ok(file_id) => file_id,
    Err(e) => {
        let error_msg = format!("Error parsing URL: {}", e);
        return handle_error(error_msg);
    }
};

let result = match api_call(&file_id).await {
    Ok(data) => data,
    Err(e) => {
        let error_msg = format!("API call failed: {}", e);
        return handle_error(error_msg);
    }
};
```

**Use let/else for pattern matching** when the error context is obvious:
```rust
let FigmaUrlType::File { file_id, node_id } = parsed.url_type else {
    return handle_error("URL must be a Figma file URL".to_string());
};
```

**Split complex expressions into multiple lines**:
```rust
// Instead of deeply nested expressions:
// return Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))]))

// Use intermediate variables:
let error_msg = format!("Error: {}", e);
let content = Content::text(error_msg);
return Ok(CallToolResult::success(vec![content]));
```

**Prefer linear flow with variable shadowing**:
```rust
// Use shadowing instead of creating new variable names for linear transformations
let result = extract_and_validate_input()?;
let result = fetch_from_api(&result).await?;
let result = process_data(result)?;

return_success(result)

// Instead of:
// let parsed = extract_and_validate_input()?;
// let data = fetch_from_api(&parsed).await?;
// let result = process_data(data)?;
// return_success(result)
```

**Add empty line before return statements** to visually separate the final result from the processing logic.

**Order file contents by visibility, keeping implementations close to struct declarations**:
```rust
// Main types and structs first
pub struct MyStruct {
    // ...
}

// Keep implementations immediately after their struct declarations
impl MyStruct {
    pub fn public_method() {
        // ...
    }
}

// Supporting structs and their implementations
struct HelperStruct {
    // ...
}

impl HelperStruct {
    fn helper_method() {
        // ...
    }
}

// Standalone helper functions at the bottom
fn helper_function() {
    // ...
}
```

This approach reduces cognitive load and makes the "happy path" easier to follow by eliminating deeply nested match expressions and long lines.

## Environment Setup

Required environment variable:
- `FIGMA_TOKEN`: Personal access token from Figma Developer Settings

Optional:
- `RUST_LOG`: Set to `debug` for detailed logging of HTTP requests and tool execution