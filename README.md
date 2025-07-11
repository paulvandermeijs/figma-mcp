# Figma MCP Server

A Model Context Protocol (MCP) server that provides access to Figma files and
design assets through a standardized interface.

## Features

- **File Access**: Retrieve Figma file structures with depth control
- **Node Navigation**: Access specific components within files
- **Image Export**: Export design assets in PNG, JPG, SVG, or PDF formats
- **URL Parsing**: Extract file keys from Figma URLs
- **Depth Management**: Control response size to prevent token limits
- **Resource Support**: Exported images are available as MCP resources with
  base64-encoded content

## Installation

### Option 1: Install from crates.io (Recommended)

```bash
cargo install figma-mcp
```

### Option 2: Build from source

```bash
git clone https://github.com/paulvandermeijs/figma-mcp.git
cd figma-mcp
cargo build --release
```

## Quick Start

1. Get your Figma token from
   [Developer Settings](https://www.figma.com/developers/api#access-tokens)

2. Set environment variable:

   ```bash
   export FIGMA_TOKEN="your_token_here"
   ```

3. Run the server:

   ```bash
   # If installed via cargo install
   figma-mcp

   # If built from source
   cargo run --release
   ```

## Usage

### Workflow

1. Use `parse_figma_url` to extract file key from any Figma URL
2. Use file key with other tools to access file data
3. Use depth parameter to control response size

### Available Tools

- `parse_figma_url` - Extract file key from Figma URLs
- `get_file` - Get file structure (with depth control)
- `get_file_nodes` - Get specific nodes (with depth control)
- `export_images` - Export images from nodes
- `get_me` - Test authentication
- `help` - Usage instructions

### Resources

Exported images are automatically available as MCP resources:

- List resources to see all exported images
- Read resources to get base64-encoded image data
- Resource URIs: `figma://file/{file_key}/node/{node_id}.{format}`

### Depth Parameter

- **depth=1** (default): Pages only (files) or direct children (nodes)
- **depth=2**: Pages + top-level objects or children + grandchildren
- **depth=3+**: Deeper traversal (use carefully)

## Supported URLs

- `https://www.figma.com/file/FILE_ID/filename`
- `https://www.figma.com/design/FILE_ID/filename`
- URLs with node IDs: `?node-id=1%3A2`

## Development

```bash
cargo test          # Run tests
RUST_LOG=info cargo run  # Run with logging
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
