# MCP Developer Documentation Server

This is a Model Context Protocol (MCP) server that provides documentation for multiple development tools and frameworks:

- Rust Crates (via docs.rs)
- Tauri (from local docs)
- Svelte (from local docs)
- SvelteKit (from local docs)

The server provides essential context for LLMs when working with these technologies.

## Features

- Fetches documentation for any Rust crate available on docs.rs
- Provides searchable documentation for Tauri
- Provides searchable documentation for both Svelte and SvelteKit
- Allows searching by specific topics within each documentation set
- Strips HTML and formats the content for readability
- Limits response size to prevent overwhelming the client
- Uses the latest MCP SDK (v1.6.1)

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-dev-docs.git
cd mcp-dev-docs

# Install dependencies
npm install
```

### Prerequisites

- Node.js 
- npm 

## Usage

```bash
# Start the server directly
npm start
```

## Directory Structure

The repository includes:

- `index.js`: Main server implementation
- `OtherLangs/`: Directory containing local documentation files
  - `TAURIllms.txt`: Documentation for Tauri
  - `SVELTEllms-full.txt`: Documentation for Svelte and SvelteKit

## Integrating with AI Assistants

### Claude Desktop or Cursor

Add the following to your Claude Desktop configuration file (`claude_desktop_config.json`) or configure in Cursor:

```json
{
  "mcpServers": {
    "dev-docs": {
      "command": "node",
      "args": ["/absolute/path/to/index.js"]
    }
  }
}
```

Replace `/absolute/path/to/index.js` with the absolute path to the index.js file in this repository.

## Example Usage

Once the server is running and configured with your AI assistant, you can ask questions like:

### For Rust:
- "Look up the documentation for the 'tokio' crate"
- "What features does the 'serde' crate provide?"
- "Show me the documentation for 'ratatui'"

### For Tauri:
- "Show me Tauri documentation on window customization"
- "What are Tauri capabilities?"
- "How do I handle security in Tauri?"

### For Svelte:
- "Explain Svelte component structure"
- "How do Svelte runes work?"
- "Show me SvelteKit documentation on routing"

## Available Tools

The server implements three MCP tools:

### 1. `lookup_crate_docs`

Fetches documentation for a Rust crate from docs.rs.

Parameters:
- `crateName` (string): Name of the Rust crate to lookup

### 2. `lookup_tauri_docs`

Fetches documentation for Tauri.

Parameters:
- `topic` (string, optional): Topic in Tauri documentation to look up. Defaults to "overview".

### 3. `lookup_svelte_docs`

Fetches documentation for Svelte or SvelteKit.

Parameters:
- `topic` (string, optional): Topic to look up. Defaults to "overview".
- `type` (enum, optional): Type of documentation: 'svelte' or 'sveltekit'. Defaults to "svelte".

## Testing with MCP Inspector

You can test this server using the MCP Inspector:

```bash
npx @modelcontextprotocol/inspector
```

Then select the "Connect to a local server" option and follow the prompts.

## How It Works

The server processes documentation in different ways:

1. For Rust crates: Fetches documentation from docs.rs
2. For Tauri and Svelte/SvelteKit: Reads from local files and searches for relevant sections
3. All documentation is formatted for readability and truncated if necessary

## SDK Implementation Notes

This server uses the MCP SDK with carefully structured import paths. If you're modifying the code, be aware that:

1. The SDK requires importing from specific paths (e.g., `@modelcontextprotocol/sdk/server/mcp.js`)
2. We use the high-level McpServer API rather than the low-level tools
3. The tool definition uses Zod for parameter validation
4. Console output is redirected to stderr to avoid breaking the MCP protocol
5. The tool returns properly formatted MCP response objects

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT