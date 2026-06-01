# 🚀 MCP OpenAPI Transformer

A Model Context Protocol (MCP) server that dynamically exposes REST API endpoints as MCP tools by reading OpenAPI specifications.

## 📋 Features

- **Dynamic Tool Generation**: Automatically creates MCP tools from OpenAPI spec endpoints
- **Full OpenAPI Support**: Parses paths, parameters, request bodies, and descriptions
- **HTTP Method Support**: Handles GET, POST, PUT, DELETE, and PATCH operations
- **Flattened Parameters**: Request body properties are flattened into top-level tool parameters for better UX
- **Schema Reference Resolution**: Handles both inline schemas and `$ref` references to component schemas
- **Parameter Handling**: Supports path parameters, query parameters, and flattened request body fields
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

Optional:

- **`HEADER_<name>`**: Custom HTTP header on every request (spec fetch + tool calls). Underscores in `<name>` become hyphens.
- **`INSECURE`**: Set to `true` or `1` to accept invalid TLS certificates.

Example (dietapp with API key auth):

```bash
HEADER_AUTHORIZATION="ApiKey dk_your_secret_here" \
BASE_URL="http://localhost:5000/api/v1" \
DOC_URL="http://localhost:5000/api-docs.json" \
./target/release/mcp-openapi-transformer
```

Alternative header style:

```bash
HEADER_X_API_KEY="dk_your_secret_here" \
...
```

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

## 🎯 Parameter Flattening

Request body parameters are **automatically flattened** into top-level tool parameters for a better user experience.

### Before (Wrapped)
```json
{
  "properties": {
    "body": {
      "type": "object",
      "description": "Request body"
    }
  }
}
```

### After (Flattened) ✅
```json
{
  "properties": {
    "name": { "type": "string" },
    "photoUrls": { "type": "array" },
    "status": { "type": "string" },
    "category": { "type": "string" }
  },
  "required": ["name", "photoUrls"]
}
```

### Benefits
- ✨ Clear visibility of required fields
- ✨ Type information for each parameter
- ✨ Better IDE autocomplete support
- ✨ Improved validation and documentation

The server automatically:
1. Resolves schema references (`$ref: "#/components/schemas/..."`)
2. Extracts all properties from the request body schema
3. Adds them as individual parameters with proper types
4. Marks required fields appropriately
5. Reconstructs the request body automatically during execution

## 🧪 Testing

### Run Parameter Flattening Test

Test with the public Petstore API:

```bash
python3 test_flattening.py
```

This will:
- ✅ Connect to Petstore API
- ✅ Verify no `body` parameters in POST endpoints
- ✅ Confirm parameters are properly flattened
- ✅ Test real API calls with flattened parameters
- ✅ Display detailed test results

### Run the E2E Test

Make sure your API is running on `localhost:5000`, then:

```bash
python3 test.py
```

This will:
- ✅ Start the MCP server
- ✅ Initialize the MCP session (with proper handshake)
- ✅ List all available tools from your OpenAPI spec
- ✅ Find meal-related tools
- ✅ Call tools to get meals from yesterday
- ✅ Display results with pretty formatting

### Test with Different API

Edit the URLs at the top of `test.py`:

```python
BASE_URL = "http://your-api.com/v1"
DOC_URL = "http://your-api.com/openapi.json"
```

### Manual Testing

Start the server manually:

```bash
BASE_URL="http://localhost:5000/api/v1" \
DOC_URL="http://localhost:5000/api-docs.json" \
RUST_LOG="info" \
./target/release/mcp-openapi-transformer
```

Then send JSON-RPC messages via stdin:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_meals","arguments":{"date":"2025-11-21"}}}
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

- [ ] Support for authentication (API keys, OAuth, etc.) beyond env headers
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
- Auth via `HEADER_*` env vars only (no OpenAPI `securitySchemes` auto-mapping yet)

---

**Built with 🦀 Rust and the MCP SDK**
