# 🧪 Testing Guide

This document explains how to test the MCP OpenAPI Transformer server.

## Prerequisites

1. **Build the server** (if not already built):
   ```bash
   unset ARGV0 && cargo build --release
   ```

2. **Have your API running** on `http://localhost:5000` with:
   - OpenAPI spec at: `http://localhost:5000/api-docs.json`
   - API endpoints at: `http://localhost:5000/api/v1`

## Automated E2E Test (Recommended)

### Quick Start

```bash
python3 test.py
```

### What It Does

The test script performs a complete end-to-end test:

1. **🚀 Starts the MCP Server**
   - Configures BASE_URL and DOC_URL
   - Launches the server process
   - Captures stdin/stdout for communication

2. **🔧 Initializes MCP Session**
   - Sends initialize request
   - Verifies server info
   - Displays server capabilities

3. **📋 Lists All Tools**
   - Requests tools/list
   - Displays all available endpoints as tools
   - Identifies meal-related tools

4. **🍽️ Tests Meal Retrieval**
   - Calculates yesterday's date
   - Finds appropriate tool (meal-related GET endpoint)
   - Calls the tool with date parameters
   - Displays results

5. **✅ Validates Results**
   - Pretty-prints JSON responses
   - Shows success/error status
   - Counts returned items

### Expected Output

```
🚀 Starting MCP OpenAPI Transformer E2E Test
============================================================
📍 Base URL: http://localhost:5000/api/v1
📄 OpenAPI Doc: http://localhost:5000/api-docs.json

============================================================
  Starting MCP Server
============================================================

✅ Server started successfully

============================================================
  Initializing MCP Session
============================================================

📤 Sending: {"jsonrpc":"2.0","id":1,"method":"initialize",...}
📥 Received: {"jsonrpc":"2.0","id":1,"result":{...}}
✅ Server Info:
   Name: mcp-openapi-transformer
   Version: 0.1.0
   Instructions: This server exposes REST API endpoints...

============================================================
  Listing Available Tools
============================================================

✅ Found 15 tools:

  1. get_meals
     Get all meals
     🍽️  ** MEAL-RELATED TOOL **

  2. post_meals
     Create a new meal
     🍽️  ** MEAL-RELATED TOOL **
  ...

============================================================
  Testing Meal Retrieval
============================================================

🗓️  Target date: 2025-11-21 (yesterday)
✅ Selected tool: get_meals

📋 Tool details:
   Name: get_meals
   Description: Get all meals

   Parameters:
      - date (string) [✓ REQUIRED]
        Filter meals by date (YYYY-MM-DD)

   Setting date = 2025-11-21

🔧 Calling tool: get_meals
   Arguments: {
     "date": "2025-11-21"
   }

============================================================
  Tool Result
============================================================

✅ Tool executed successfully!

📄 Response:
────────────────────────────────────────────────────────────
[
  {
    "id": 1,
    "name": "Breakfast",
    "date": "2025-11-21",
    "calories": 450
  },
  {
    "id": 2,
    "name": "Lunch",
    "date": "2025-11-21",
    "calories": 680
  }
]
────────────────────────────────────────────────────────────
📊 Total items: 2
────────────────────────────────────────────────────────────

============================================================
  Test Complete
============================================================
✅ All tests completed successfully!
```

## Manual Testing

### Using the Manual Test Script

```bash
./manual_test.sh
```

This starts the server and waits for JSON-RPC input. You can then paste commands:

### Example Commands

**1. Initialize:**
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
```

**2. List Tools:**
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

**3. Call a Tool:**
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_meals","arguments":{"date":"2025-11-21"}}}
```

## Testing with Different APIs

To test with a different API, edit the configuration at the top of `test.py`:

```python
BASE_URL = "http://your-api.com/v1"
DOC_URL = "http://your-api.com/openapi.json"
```

## Troubleshooting

### Server Not Starting

**Error:** `Binary not found`
```bash
# Build the project first
unset ARGV0 && cargo build --release
```

### API Not Responding

**Error:** `Failed to fetch OpenAPI spec`
```bash
# Check if your API is running
curl http://localhost:5000/api-docs.json
```

### No Meal Tools Found

The test will automatically:
1. Look for tools with "meal" in name/description
2. Fall back to any GET endpoints
3. Display available tools for manual inspection

### Connection Issues

**Error:** `No response from server`
- Check server logs in stderr
- Verify JSON-RPC format
- Ensure proper line endings (\n)

## Debug Mode

Enable verbose logging:

```bash
RUST_LOG=debug python3 test_e2e.py
```

This will show:
- All HTTP requests to the API
- OpenAPI parsing details
- Tool generation process
- Parameter mapping

## MCP Protocol Reference

The test uses JSON-RPC 2.0 over stdio:

### Message Format
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "method_name",
  "params": { ... }
}
```

### Supported Methods
- `initialize` - Start MCP session
- `tools/list` - Get available tools
- `tools/call` - Execute a tool

### Response Format
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

## Additional Test Scenarios

### Test All Tools
```python
# Modify test_e2e.py to iterate through all tools
for tool in tools:
    try:
        result = client.call_tool(tool['name'], {})
        print(f"✅ {tool['name']} works")
    except:
        print(f"❌ {tool['name']} failed")
```

### Test Error Handling
```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"invalid_tool","arguments":{}}}
```

### Test Parameter Validation
```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_meals","arguments":{"invalid_param":"value"}}}
```

---

**Happy Testing! 🎉**
