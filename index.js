// index.js - Plain JavaScript version
const axios = require('axios');
const { convert: htmlToText } = require('html-to-text');

// Import specific modules from the SDK with corrected paths
const { McpServer } = require('@modelcontextprotocol/sdk/server/mcp.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { z } = require('zod');

// Redirect console.log to stderr to avoid breaking the MCP protocol
const originalConsoleLog = console.log;
console.log = function() {
  console.error.apply(console, arguments);
};

// Initialize the MCP server
const server = new McpServer({
  name: 'rust-docs',
  version: '1.0.0'
});

// Define tool with proper Zod schema for parameters
server.tool(
  'lookup_crate_docs',
  'Lookup documentation for a Rust crate from docs.rs',
  { crateName: z.string().optional().describe('Name of the Rust crate to lookup documentation for') },
  async (args) => {
    try {
      // Extract crateName from args or use default
      const crateName = args.crateName || "tokio";
      
      console.error(`Fetching documentation for default crate: ${crateName}`);
      
      // Construct the docs.rs URL for the crate
      const url = `https://docs.rs/${crateName}/latest/${crateName}/index.html`;
      console.error(`Making request to: ${url}`);
      
      // Fetch the HTML content
      const response = await axios.get(url);
      console.error(`Received response with status: ${response.status}`);
      
      // Convert HTML to text
      const text = htmlToText(response.data, {
        wordwrap: 130,
        selectors: [
          { selector: 'a', options: { ignoreHref: true } },
          { selector: 'img', format: 'skip' }
        ]
      });
      
      // Truncate if necessary
      const maxLength = 8000;
      const truncatedText = text.length > maxLength 
        ? text.substring(0, maxLength) + `\n\n[Content truncated. Full documentation available at ${url}]` 
        : text;
      
      console.error(`Successfully processed docs for ${crateName}`);
      return {
        content: [{ type: "text", text: truncatedText }]
      };
    } catch (error) {
      console.error(`Error fetching documentation:`, error.message);
      return {
        content: [{ type: "text", text: `Error: Could not fetch documentation. ${error.message}` }],
        isError: true
      };
    }
  }
);

// Define prompts for tools
server.prompt(
  'lookup_crate_docs',
  { crateName: z.string().describe('Name of the Rust crate to lookup documentation for') },
  ({ crateName }) => ({
    messages: [
      {
        role: "user",
        content: {
          type: "text",
          text: `Please analyze and summarize the documentation for the Rust crate '${crateName}'. Focus on:
1. The main purpose and features of the crate
2. Key types and functions
3. Common usage patterns
4. Any important notes or warnings
5. VERY IMPORTANT: Latest Version

Documentation content will follow.`
        }
      }
    ]
  })
);

// Connect to the stdio transport and start the server
server.connect(new StdioServerTransport())
  .then(() => {
    console.error('MCP Crate Docs Server is running...');
  })
  .catch((err) => {
    console.error('Failed to start MCP server:', err);
    process.exit(1);
  }); 