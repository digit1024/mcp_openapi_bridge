#!/usr/bin/env python3
"""
Test for verifying parameter flattening works correctly.
Tests with Petstore API which has POST endpoints with request bodies.
"""

import subprocess
import json
import time
import os
import sys
import select

# Configuration for Petstore API
BASE_URL = "https://petstore3.swagger.io/api/v3"
DOC_URL = "https://petstore3.swagger.io/api/v3/openapi.json"

print("🧪 MCP OpenAPI Transformer - Parameter Flattening Test")
print("="*70)
print(f"📍 Base URL: {BASE_URL}")
print(f"📄 OpenAPI Doc: {DOC_URL}")
print()

# Setup environment
env = os.environ.copy()
env["BASE_URL"] = BASE_URL
env["DOC_URL"] = DOC_URL
env["RUST_LOG"] = "info"

# Start server
print("Starting MCP server...")
proc = subprocess.Popen(
    ["./target/release/mcp-openapi-transformer"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=1,
    env=env
)

time.sleep(3)  # Give it time to fetch and parse OpenAPI spec

def send_request(method, params=None, req_id=None):
    """Send JSON-RPC request"""
    global next_id
    if req_id is None:
        req_id = next_id
        next_id += 1
    
    req = {"jsonrpc": "2.0", "id": req_id, "method": method}
    if params:
        req["params"] = params
    
    print(f"📤 {method}")
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    
    ready, _, _ = select.select([proc.stdout], [], [], 5.0)
    if not ready:
        print(f"❌ No response after 5 seconds!")
        return None
    
    response = proc.stdout.readline()
    if not response:
        return None
    
    return json.loads(response)

def send_notification(method, params=None):
    """Send JSON-RPC notification"""
    notif = {"jsonrpc": "2.0", "method": method}
    if params:
        notif["params"] = params
    
    print(f"📤 {method} (notification)")
    proc.stdin.write(json.dumps(notif) + "\n")
    proc.stdin.flush()

next_id = 1

# Initialize
print("\n" + "="*70)
print("Step 1: Initialize")
print("="*70)

resp = send_request("initialize", {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {"name": "test", "version": "1.0"}
})

if not resp or "error" in resp:
    print(f"❌ Initialize failed: {resp}")
    proc.terminate()
    sys.exit(1)

server_info = resp.get("result", {}).get("serverInfo", {})
instructions = resp.get("result", {}).get("instructions", "")

print(f"✅ Connected to: {server_info.get('name')} v{server_info.get('version')}")
print(f"   {instructions}")

# Send initialized notification
time.sleep(0.5)
send_notification("notifications/initialized")
time.sleep(0.5)

# List tools
print("\n" + "="*70)
print("Step 2: List Tools")
print("="*70)

resp = send_request("tools/list")
if not resp or "error" in resp:
    print(f"❌ Failed: {resp}")
    proc.terminate()
    sys.exit(1)

tools = resp.get("result", {}).get("tools", [])
print(f"✅ Found {len(tools)} tools")
print()

# Find POST endpoints (they should have flattened body parameters)
post_tools = [t for t in tools if t.get("name", "").startswith("post_")]

print(f"📝 Found {len(post_tools)} POST endpoints")
print()

# Analyze parameter structure
print("="*70)
print("Step 3: Verify Parameter Flattening")
print("="*70)
print()

test_results = []

for tool in post_tools[:5]:  # Test first 5 POST endpoints
    name = tool.get("name", "")
    input_schema = tool.get("inputSchema", {})
    properties = input_schema.get("properties", {})
    required = input_schema.get("required", [])
    
    print(f"🔍 Tool: {name}")
    print(f"   Description: {tool.get('description', 'N/A')}")
    
    # Check if there's a "body" parameter (old style)
    has_body_param = "body" in properties
    
    if has_body_param:
        print(f"   ❌ FAIL: Still has 'body' parameter (not flattened)")
        test_results.append(("FAIL", name, "Has body parameter"))
    else:
        print(f"   ✅ PASS: No 'body' parameter found")
        
        # Show all parameters
        if properties:
            print(f"   📋 Parameters ({len(properties)}):")
            for param_name, param_info in properties.items():
                param_type = param_info.get("type", "unknown")
                is_required = param_name in required
                marker = "✓" if is_required else "○"
                desc = param_info.get("description", "")
                print(f"      [{marker}] {param_name} ({param_type})")
                if desc and len(desc) < 60:
                    print(f"          {desc}")
            test_results.append(("PASS", name, f"{len(properties)} flattened parameters"))
        else:
            print(f"   ⚠️  WARNING: No parameters at all")
            test_results.append(("WARN", name, "No parameters"))
    
    print()

# Summary
print("="*70)
print("Step 4: Test Real API Call (POST with flattened params)")
print("="*70)
print()

# Try to find a suitable POST endpoint to test
# Let's try post_pet if it exists
post_pet = next((t for t in tools if t.get("name") == "post_pet"), None)

if post_pet:
    print("🔧 Testing: post_pet")
    input_schema = post_pet.get("inputSchema", {})
    properties = input_schema.get("properties", {})
    
    print(f"   Parameters available: {list(properties.keys())}")
    print()
    
    # Build a minimal request with flattened parameters
    # Based on Petstore schema, a pet needs: name, photoUrls
    arguments = {
        "name": "TestDog",
        "photoUrls": ["https://example.com/dog.jpg"],
        "status": "available"
    }
    
    print(f"📤 Calling post_pet with flattened parameters:")
    print(f"   {json.dumps(arguments, indent=2)}")
    print()
    
    resp = send_request("tools/call", {
        "name": "post_pet",
        "arguments": arguments
    })
    
    if not resp:
        print("❌ No response")
    elif "error" in resp:
        print(f"❌ Error: {resp['error']}")
    else:
        result = resp.get("result", {})
        is_error = result.get("isError", False)
        content = result.get("content", [])
        
        if is_error:
            print("⚠️  API returned error (this might be expected for test data):")
            for item in content:
                if item.get("type") == "text":
                    print(f"   {item.get('text')[:200]}")
        else:
            print("✅ API call succeeded!")
            for item in content:
                if item.get("type") == "text":
                    text = item.get("text", "")
                    try:
                        data = json.loads(text)
                        print("   Response:")
                        print(json.dumps(data, indent=2)[:500])
                    except:
                        print(f"   {text[:200]}")
else:
    print("⚠️  No post_pet endpoint found, skipping API call test")

# Final summary
print()
print("="*70)
print("Test Summary")
print("="*70)
print()

passed = sum(1 for r in test_results if r[0] == "PASS")
failed = sum(1 for r in test_results if r[0] == "FAIL")
warned = sum(1 for r in test_results if r[0] == "WARN")

print(f"✅ PASSED: {passed}")
print(f"❌ FAILED: {failed}")
print(f"⚠️  WARNED: {warned}")
print()

if failed > 0:
    print("Failed tests:")
    for status, name, msg in test_results:
        if status == "FAIL":
            print(f"  - {name}: {msg}")
    print()

print("="*70)
if failed == 0:
    print("🎉 All parameter flattening tests PASSED!")
else:
    print("❌ Some tests FAILED - parameters are not properly flattened")
print("="*70)

proc.terminate()
proc.wait()

sys.exit(0 if failed == 0 else 1)
