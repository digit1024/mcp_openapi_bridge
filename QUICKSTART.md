# 🚀 Quick Start Guide

## Run the E2E Test in 3 Steps

### Step 1: Build the Server
```bash
unset ARGV0 && cargo build --release
```

### Step 2: Start Your API
Make sure your API is running on:
- **Base URL:** `http://localhost:5000/api/v1`
- **OpenAPI Spec:** `http://localhost:5000/api-docs.json`

### Step 3: Run the Test
```bash
python3 test.py
```

That's it! 🎉

---

## What Gets Tested

✅ Server initialization  
✅ OpenAPI spec parsing  
✅ Dynamic tool generation  
✅ Listing all available endpoints  
✅ Calling API endpoints through MCP  
✅ Getting meals from yesterday  

---

## Using Different URLs

Edit `test_e2e.py` lines 52-53:
```python
base_url = "http://localhost:5000/api/v1"      # Change this
doc_url = "http://localhost:5000/api-docs.json" # Change this
```

Or set environment variables:
```bash
BASE_URL="http://your-api/v1" \
DOC_URL="http://your-api/openapi.json" \
python3 test_e2e.py
```

### API keys / custom headers

Any env var `HEADER_<name>=<value>` is sent on every HTTP request. Underscores in `<name>` become hyphens (`HEADER_X_API_KEY` → `X-API-Key`).

```bash
# dietapp (Authorization: ApiKey dk_…)
HEADER_AUTHORIZATION="ApiKey dk_your_key" \
BASE_URL="http://localhost:5000/api/v1" \
DOC_URL="http://localhost:5000/api-docs.json" \
./target/release/mcp-openapi-transformer
```

---

## Manual Testing

```bash
# Start server in interactive mode
./manual_test.sh

# In another terminal, send commands:
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./manual_test.sh

echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./manual_test.sh
```

---

## Using with MCP Inspector

```bash
# Start with MCP inspector (if installed)
npx @modelcontextprotocol/inspector \
  BASE_URL=http://localhost:5000/api/v1 \
  DOC_URL=http://localhost:5000/api-docs.json \
  ./target/release/mcp-openapi-transformer
```

---

For more details, see:
- 📖 **README.md** - Full documentation
- 🧪 **TESTING.md** - Detailed testing guide
- 🔧 **example.sh** - Petstore API example
