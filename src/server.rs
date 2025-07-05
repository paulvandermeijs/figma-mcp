use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    Error as McpError, ServerHandler, ServiceExt,
};
use serde::Deserialize;
use std::future::Future;

use crate::{
    figma::{FigmaClient, FigmaUrlParser},
    Error,
};

#[derive(Clone)]
pub struct FigmaServer {
    client: FigmaClient,
    url_parser: FigmaUrlParser,
    tool_router: ToolRouter<FigmaServer>,
}

#[tool_router]
impl FigmaServer {
    pub fn new(figma_token: String) -> std::result::Result<Self, Error> {
        let client = FigmaClient::new(figma_token)?;
        let url_parser = FigmaUrlParser::new();

        Ok(Self {
            client,
            url_parser,
            tool_router: Self::tool_router(),
        })
    }

    pub async fn run_stdio(self) -> std::result::Result<(), Error> {
        tracing::info!("Starting Figma MCP server");

        let service = self.serve(stdio()).await.map_err(|e| {
            tracing::error!("Failed to start MCP service: {:?}", e);
            Error::Mcp(e.into())
        })?;

        tracing::info!("MCP service started successfully, waiting for connections");
        service.waiting().await.map_err(|e| {
            tracing::error!("MCP service error: {:?}", e);
            Error::Mcp(e.into())
        })?;

        Ok(())
    }

    #[tool(description = "Parse a Figma URL to extract IDs and determine the URL type")]
    async fn parse_figma_url(
        &self,
        Parameters(ParseUrlRequest { url }): Parameters<ParseUrlRequest>,
    ) -> Result<CallToolResult, McpError> {
        let url_info = match self.url_parser.parse(&url) {
            Ok(parsed) => parsed,
            Err(e) => {
                let error_msg = format!("Error parsing URL: {}", e);
                return tool_error(error_msg);
            }
        };

        let result = serde_json::to_string_pretty(&url_info)
            .unwrap_or_else(|e| format!("Serialization error: {}", e));

        tool_success(result)
    }

    #[tool(description = "Get file contents from a Figma file using file key")]
    async fn get_file(
        &self,
        Parameters(GetFileRequest { file_key, depth }): Parameters<GetFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        let depth = depth.unwrap_or(1);
        let result = match self.client.get_file(&file_key, Some(depth)).await {
            Ok(file) => file,
            Err(e) => {
                let error_msg = format!("Error fetching file: {}", e);
                return tool_error(error_msg);
            }
        };

        let result = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("Serialization error: {}", e));

        tool_success(result)
    }

    #[tool(description = "Get specific nodes from a file using file key")]
    async fn get_file_nodes(
        &self,
        Parameters(GetFileNodesRequest {
            file_key,
            node_ids,
            depth,
        }): Parameters<GetFileNodesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let node_ids: Vec<String> = node_ids.split(',').map(|s| s.trim().to_string()).collect();
        let depth = depth.unwrap_or(1);

        let result = match self
            .client
            .get_file_nodes(&file_key, &node_ids, Some(depth))
            .await
        {
            Ok(nodes) => nodes,
            Err(e) => {
                let error_msg = format!("Error fetching file nodes: {}", e);
                return tool_error(error_msg);
            }
        };

        let result = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("Serialization error: {}", e));

        tool_success(result)
    }

    #[tool(description = "Export images from a Figma file using file key")]
    async fn export_images(
        &self,
        Parameters(ExportImageRequest {
            file_key,
            node_ids,
            format,
            scale,
        }): Parameters<ExportImageRequest>,
    ) -> Result<CallToolResult, McpError> {
        let node_ids_to_export: Vec<String> =
            node_ids.split(',').map(|s| s.trim().to_string()).collect();

        let format = format.as_deref().unwrap_or("png");
        let result = match self
            .client
            .export_images(&file_key, &node_ids_to_export, format, scale)
            .await
        {
            Ok(export_result) => export_result,
            Err(e) => {
                let error_msg = format!("Error exporting images: {}", e);
                return tool_error(error_msg);
            }
        };

        let result = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("Serialization error: {}", e));

        tool_success(result)
    }

    #[tool(description = "Get current user information (useful for testing authentication)")]
    async fn get_me(&self) -> Result<CallToolResult, McpError> {
        let result = match self.client.get_me().await {
            Ok(user) => user,
            Err(e) => {
                let error_msg = format!("Error fetching user info: {}", e);
                return tool_error(error_msg);
            }
        };

        let result = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("Serialization error: {}", e));

        tool_success(result)
    }

    #[tool(description = "Help: How to use this Figma file MCP server")]
    async fn help(&self) -> Result<CallToolResult, McpError> {
        let help_text = r#"
# Figma MCP Server Help

This MCP server provides tools to access and work with Figma files using file keys with depth control to manage response size.

## Workflow

1. First, use `parse_figma_url` to extract the file key from a Figma URL
2. Then use the file key with other tools to access file data
3. Use the depth parameter to control how much data is returned and avoid token limits
4. Navigate deeper into the file structure using recursive calls with specific node IDs

## Available Tools

### URL Parsing
- `parse_figma_url`: Parse any Figma URL to extract file key and node information

### File Operations (require file key from parse_figma_url)
- `get_file`: Get file structure using file key with depth control (default: 1)
- `get_file_nodes`: Get specific nodes using file key with depth control (default: 1)
- `export_images`: Export images from file using file key
- `get_me`: Test authentication and get user info

## Depth Parameter

Both `get_file` and `get_file_nodes` support a depth parameter to limit response size:

- **depth=1** (default): For files: pages only. For nodes: direct children only
- **depth=2**: For files: pages + top-level objects. For nodes: children + grandchildren
- **depth=3+**: Deeper traversal (use carefully to avoid large responses)

## Recursive Navigation Strategy

To navigate large files without exceeding token limits:

1. Start with `get_file` at depth=1 to see page structure
2. Use `get_file_nodes` with specific page IDs at depth=1 to explore page contents
3. Use `get_file_nodes` with specific component/frame IDs for deeper inspection
4. Adjust depth as needed based on response size

## Supported URL Formats
- File: https://www.figma.com/file/FILE_ID/filename
- File with node: https://www.figma.com/file/FILE_ID/filename?node-id=1%3A2
- Design URL: https://www.figma.com/design/FILE_ID/filename

## Authentication
Set your Figma personal access token as an environment variable:
export FIGMA_TOKEN="your_figma_token_here"

Get your token from: https://www.figma.com/developers/api#access-tokens
"#;

        Ok(CallToolResult::success(vec![Content::text(
            help_text.to_string(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for FigmaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation::from_build_env(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("A Figma MCP server that provides tools to access Figma files and export images. Use 'help' tool for usage instructions.".into()),
        }
    }
}

// Parameter structs for MCP tools
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ParseUrlRequest {
    #[schemars(description = "The Figma URL to parse (file or design URL)")]
    pub url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetFileRequest {
    #[schemars(description = "The Figma file key (extract from URL using parse_figma_url)")]
    pub file_key: String,
    #[schemars(
        description = "Depth to traverse into the document tree (default: 1). Use 1 for pages only, 2 for pages + top-level objects, etc."
    )]
    pub depth: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExportImageRequest {
    #[schemars(description = "The Figma file key (extract from URL using parse_figma_url)")]
    pub file_key: String,
    #[schemars(description = "Comma-separated node IDs to export")]
    pub node_ids: String,
    #[schemars(description = "Export format: png, jpg, svg, OR pdf")]
    pub format: Option<String>,
    #[schemars(description = "Export scale factor (1.0, 2.0, 4.0)")]
    pub scale: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetFileNodesRequest {
    #[schemars(description = "The Figma file key (extract from URL using parse_figma_url)")]
    pub file_key: String,
    #[schemars(description = "Comma-separated list of node IDs to fetch")]
    pub node_ids: String,
    #[schemars(
        description = "Depth to traverse from each node (default: 1). Use 1 for direct children only, 2 for children + grandchildren, etc."
    )]
    pub depth: Option<u32>,
}

// Helper functions
fn tool_error(message: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![Content::text(message)]))
}

fn tool_success(content: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(content)]))
}
