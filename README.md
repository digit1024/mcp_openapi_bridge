# 🚀 MCP OpenAPI Transformer

A Model Context Protocol (MCP) server that dynamically exposes REST API endpoints as MCP tools by reading OpenAPI specifications.

## 📋 Features

- **Dynamic Tool Generation**: Automatically creates MCP tools from OpenAPI spec endpoints
- **Full OpenAPI Support**: Parses paths, parameters, request bodies, and descriptions
- **HTTP Method Support**: Handles GET, POST, PUT, DELETE, and PATCH operations
- **Parameter Handling**: Supports path parameters, query parameters, and request bodies
- **Error Handling**: Comprehensive error reporting for API calls

## 🏗️ Architecture

```
┌─────────────┐
│   MCP       │
│   Client    │
└──────┬──────┘
       │ MCP Protocol
       ▼
┌─────────────────────────────┐
│  MCP OpenAPI Transformer    │
│  ┌───────────────────────┐  │
│  │  Fetch OpenAPI Spec   │  │
│  └───────────┬───────────┘  │
│              ▼              │
│  ┌───────────────────────┐  │
│  │  Generate MCP Tools   │  │
│  └───────────┬───────────┘  │
│              ▼              │
│  ┌───────────────────────┐  │
│  │  Execute API Calls    │  │
│  └───────────────────────┘  │
└──────────────┬──────────────┘
               │ HTTP Requests
               ▼
         ┌──────────┐
         │   API    │
         │  Server  │
         └──────────┘
```

## 🛠️ Configuration

The server requires two environment variables:

- **`BASE_URL`**: The base URL of the API to call (e.g., `https://api.example.com`)
- **`DOC_URL`**: The URL to the OpenAPI specification JSON (e.g., `https://api.example.com/openapi.json`)

## 🚀 Usage

### Building

```bash
unset ARGV0 && cargo build --release
```

### Running

```bash
BASE_URL="https://api.example.com" \
DOC_URL="https://api.example.com/openapi.json" \
./target/release/mcp-openapi-transformer
```

### Example with Public API

Using the Petstore API as an example:

```bash
BASE_URL="https://petstore3.swagger.io/api/v3" \
DOC_URL="https://petstore3.swagger.io/api/v3/openapi.json" \
./target/release/mcp-openapi-transformer
```

## 🔧 How It Works

1. **Initialization**: The server fetches and parses the OpenAPI specification from `DOC_URL`
2. **Tool Generation**: Each endpoint in the OpenAPI spec becomes an MCP tool:
   - Tool name format: `{method}_{path}` (e.g., `get_users_id`, `post_orders`)
   - Description: Extracted from operation summary or description
   - Parameters: Generated from path parameters, query parameters, and request body
3. **Execution**: When a tool is called:
   - Path parameters are substituted into the URL
   - Query parameters are added to the request
   - Request body (if present) is sent as JSON
   - Response is returned to the MCP client

## 📝 Example Tool

Given this OpenAPI endpoint:

```json
{
  "paths": {
    "/users/{id}": {
      "get": {
        "summary": "Get user by ID",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "string" }
          }
        ]
      }
    }
  }
}
```

The server generates this MCP tool:

- **Name**: `get_users_id`
- **Description**: "Get user by ID"
- **Parameters**:
  ```json
  {
    "type": "object",
    "properties": {
      "id": {
        "type": "string"
      }
    },
    "required": ["id"]
  }
  ```

## 🧪 Testing

You can test the server using any MCP client. The server communicates via stdin/stdout using the JSON-RPC protocol.

### Manual Testing

Send JSON-RPC messages to stdin:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_users_id","arguments":{"id":"123"}}}
```

## 🔍 Logging

Set the `RUST_LOG` environment variable to control logging:

```bash
RUST_LOG=info BASE_URL="..." DOC_URL="..." ./target/release/mcp-openapi-transformer
```

Levels: `trace`, `debug`, `info`, `warn`, `error`

## 📦 Dependencies

- **mcp-sdk**: Model Context Protocol SDK for Rust
- **openapiv3**: OpenAPI v3 specification parser
- **reqwest**: HTTP client for making API calls
- **tokio**: Async runtime
- **serde/serde_json**: JSON serialization
- **anyhow**: Error handling
- **tracing**: Logging framework

## 🤝 Contributing

Contributions are welcome! This is a starting point that can be extended with:

- [ ] Support for authentication (API keys, OAuth, etc.)
- [ ] Custom headers configuration
- [ ] Response schema validation
- [ ] OpenAPI v2 (Swagger) support
- [ ] File upload/download support
- [ ] Webhook handling
- [ ] Rate limiting

## 📄 License

MIT License - feel free to use and modify!

## 🐛 Known Limitations

- Currently only supports OpenAPI v3.x specifications in JSON format
- Reference resolution (`$ref`) is limited
- Complex parameter schemas are simplified to basic types
- No authentication support yet (add headers manually if needed)

---

**Built with 🦀 Rust and the MCP SDK**
