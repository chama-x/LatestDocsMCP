// index.js - Plain JavaScript version
const axios = require('axios');
const { convert: htmlToText } = require('html-to-text');
const fs = require('fs');
const path = require('path');

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
  name: 'dev-docs',
  version: '1.0.0'
});

// Define paths to local documentation files
const DOCS_PATHS = {
  tauri: path.join(__dirname, 'OtherLangs', 'TAURIllms.txt'),
  svelte: path.join(__dirname, 'OtherLangs', 'SVELTEllms-full.txt')
};

// Helper function to read local documentation files
async function readLocalDocsFile(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch (error) {
    console.error(`Error reading file: ${filePath}`, error.message);
    throw error;
  }
}

// Define tool with proper Zod schema for parameters
server.tool(
  'lookup_crate_docs',
  'Lookup documentation for a Rust crate from docs.rs',
  { crateName: z.string().describe('Name of the Rust crate to lookup documentation for') },
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

// Tool to fetch Tauri documentation
server.tool(
  'lookup_tauri_docs',
  'Lookup documentation for Tauri',
  { topic: z.string().optional().describe('Topic in Tauri documentation to look up') },
  async (args) => {
    try {
      const topic = args.topic || "overview";
      console.error(`Fetching Tauri documentation for topic: ${topic}`);
      
      // Read the documentation file
      const docsContent = await readLocalDocsFile(DOCS_PATHS.tauri);
      
      let result = docsContent;
      
      // If topic is specified, try to find relevant sections
      if (topic && topic !== "overview") {
        const lowerTopic = topic.toLowerCase();
        const sections = docsContent.split(/^# |\n# /m);
        
        // Try to find topic in sections
        const relevantSections = sections.filter(section => 
          section.toLowerCase().includes(lowerTopic)
        );
        
        if (relevantSections.length > 0) {
          result = relevantSections.join("\n\n# ");
        } else {
          // If topic not found as section, search for mentions
          const paragraphs = docsContent.split(/\n\n/);
          const relevantParagraphs = paragraphs.filter(para => 
            para.toLowerCase().includes(lowerTopic)
          );
          
          if (relevantParagraphs.length > 0) {
            result = `# Search Results for "${topic}"\n\n${relevantParagraphs.join("\n\n")}`;
          }
        }
      }
      
      // Truncate if necessary
      const maxLength = 8000;
      const truncatedText = result.length > maxLength 
        ? result.substring(0, maxLength) + `\n\n[Content truncated. Full documentation available in Tauri docs]` 
        : result;
      
      console.error(`Successfully processed Tauri docs for topic: ${topic}`);
      return {
        content: [{ type: "text", text: truncatedText }]
      };
    } catch (error) {
      console.error(`Error fetching Tauri documentation:`, error.message);
      return {
        content: [{ type: "text", text: `Error: Could not fetch Tauri documentation. ${error.message}` }],
        isError: true
      };
    }
  }
);

// Tool to fetch Svelte documentation
server.tool(
  'lookup_svelte_docs',
  'Lookup documentation for Svelte',
  { 
    topic: z.string().optional().describe('Topic in Svelte documentation to look up'),
    type: z.enum(['svelte', 'sveltekit']).optional().describe('Type of documentation: svelte or sveltekit')
  },
  async (args) => {
    try {
      const topic = args.topic || "overview";
      const type = args.type || "svelte";
      console.error(`Fetching ${type} documentation for topic: ${topic}`);
      
      // Read the documentation file
      const docsContent = await readLocalDocsFile(DOCS_PATHS.svelte);
      
      // Determine which part of the documentation to search in
      let relevantContent;
      if (type === 'sveltekit') {
        // Extract SvelteKit section (assuming it comes after "# Start of SvelteKit documentation")
        const kitIndex = docsContent.indexOf("# Start of SvelteKit documentation");
        relevantContent = kitIndex > -1 ? docsContent.substring(kitIndex) : docsContent;
      } else {
        // Extract Svelte section (assuming it's before "# Start of SvelteKit documentation")
        const kitIndex = docsContent.indexOf("# Start of SvelteKit documentation");
        relevantContent = kitIndex > -1 ? docsContent.substring(0, kitIndex) : docsContent;
      }
      
      let result = relevantContent;
      
      // If topic is specified, try to find relevant sections
      if (topic && topic !== "overview") {
        const lowerTopic = topic.toLowerCase();
        const sections = relevantContent.split(/^# |\n# /m);
        
        // Try to find topic in sections
        const relevantSections = sections.filter(section => 
          section.toLowerCase().includes(lowerTopic)
        );
        
        if (relevantSections.length > 0) {
          result = relevantSections.join("\n\n# ");
        } else {
          // If topic not found as section, search for mentions
          const paragraphs = relevantContent.split(/\n\n/);
          const relevantParagraphs = paragraphs.filter(para => 
            para.toLowerCase().includes(lowerTopic)
          );
          
          if (relevantParagraphs.length > 0) {
            result = `# Search Results for "${topic}" in ${type}\n\n${relevantParagraphs.join("\n\n")}`;
          }
        }
      }
      
      // Truncate if necessary
      const maxLength = 8000;
      const truncatedText = result.length > maxLength 
        ? result.substring(0, maxLength) + `\n\n[Content truncated. Full documentation available in ${type} docs]` 
        : result;
      
      console.error(`Successfully processed ${type} docs for topic: ${topic}`);
      return {
        content: [{ type: "text", text: truncatedText }]
      };
    } catch (error) {
      console.error(`Error fetching ${args.type || "Svelte"} documentation:`, error.message);
      return {
        content: [{ type: "text", text: `Error: Could not fetch ${args.type || "Svelte"} documentation. ${error.message}` }],
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

server.prompt(
  'lookup_tauri_docs',
  { topic: z.string().describe('Topic in Tauri documentation to look up') },
  ({ topic }) => ({
    messages: [
      {
        role: "user",
        content: {
          type: "text",
          text: `Please analyze and summarize the Tauri documentation for '${topic}'. Focus on:
1. The main concepts and features
2. Key APIs and functionality
3. Common usage patterns
4. Any important notes or warnings
5. Latest Version information

Documentation content will follow.`
        }
      }
    ]
  })
);

server.prompt(
  'lookup_svelte_docs',
  { 
    topic: z.string().describe('Topic in Svelte/SvelteKit documentation to look up'),
    type: z.enum(['svelte', 'sveltekit']).describe('Type of documentation: svelte or sveltekit')
  },
  ({ topic, type }) => ({
    messages: [
      {
        role: "user",
        content: {
          type: "text",
          text: `Please analyze and summarize the ${type === 'sveltekit' ? 'SvelteKit' : 'Svelte'} documentation for '${topic}'. Focus on:
1. The main concepts and features
2. Key APIs and functionality
3. Common usage patterns
4. Any important notes or warnings
5. Latest Version information

Documentation content will follow.`
        }
      }
    ]
  })
);

// Connect to the stdio transport and start the server
server.connect(new StdioServerTransport())
  .then(() => {
    console.error('MCP Documentation Server is running...');
  })
  .catch((err) => {
    console.error('Failed to start MCP server:', err);
    process.exit(1);
  }); 