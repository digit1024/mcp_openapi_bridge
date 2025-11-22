#!/usr/bin/env python3
"""
E2E test for localhost meal API
Configure your API at http://localhost:5000
"""

import subprocess
import json
import time
import os
import sys
import select
from datetime import datetime, timedelta

# Configuration for localhost meal API
BASE_URL = "http://localhost:5000/api/v1"
DOC_URL = "http://localhost:5000/api-docs.json"

print("🚀 MCP OpenAPI Transformer - E2E Test")
print("="*60)
print(f"📍 Base URL: {BASE_URL}")
print(f"📄 OpenAPI Doc: {DOC_URL}")
print()
print("⚠️  Make sure your API is running on localhost:5000!")
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

time.sleep(2)

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
    
    ready, _, _ = select.select([proc.stdout], [], [], 3.0)
    if not ready:
        print(f"❌ No response!")
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
print("\n" + "="*60)
print("Step 1: Initialize")
print("="*60)

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
print("\n" + "="*60)
print("Step 2: List Tools")
print("="*60)

resp = send_request("tools/list")
if not resp or "error" in resp:
    print(f"❌ Failed: {resp}")
    proc.terminate()
    sys.exit(1)

tools = resp.get("result", {}).get("tools", [])
print(f"✅ Found {len(tools)} tools:")
print()

meal_tools = []
for i, tool in enumerate(tools, 1):
    name = tool.get("name", "")
    desc = tool.get("description", "")
    print(f"  {i:2}. {name}")
    if desc:
        print(f"      {desc}")
    
    if "meal" in name.lower() or "meal" in desc.lower():
        meal_tools.append(tool)
        print(f"      🍽️  ** MEAL TOOL **")
    print()

# Test meal tool
print("="*60)
print("Step 3: Get Meals from Yesterday")
print("="*60)

yesterday = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")
print(f"🗓️  Date: {yesterday}")
print()

if not meal_tools:
    print("⚠️  No meal-specific tools found.")
    print("   Listing all GET tools:")
    get_tools = [t for t in tools if t.get("name", "").startswith("get_")]
    for t in get_tools:
        print(f"   - {t.get('name')}")
    
    if get_tools:
        selected_tool = get_tools[0]
    else:
        print("❌ No suitable tools found")
        proc.terminate()
        sys.exit(0)
else:
    selected_tool = meal_tools[0]

tool_name = selected_tool["name"]
print(f"🔧 Using tool: {tool_name}")

# Build arguments
input_schema = selected_tool.get("inputSchema", {})
properties = input_schema.get("properties", {})
arguments = {}

print(f"\n   Parameters:")
for param_name, param_info in properties.items():
    param_type = param_info.get("type", "unknown")
    is_required = param_name in input_schema.get("required", [])
    marker = "✓" if is_required else "○"
    print(f"   [{marker}] {param_name} ({param_type})")
    
    # Auto-fill date parameters
    if "date" in param_name.lower():
        arguments[param_name] = yesterday
        print(f"       → Setting to: {yesterday}")

print(f"\n📤 Calling: {tool_name}")
print(f"   Args: {json.dumps(arguments, indent=2)}")

resp = send_request("tools/call", {
    "name": tool_name,
    "arguments": arguments
})

print("\n" + "="*60)
print("Result")
print("="*60)

if not resp:
    print("❌ No response")
elif "error" in resp:
    print(f"❌ Error: {resp['error']}")
else:
    result = resp.get("result", {})
    is_error = result.get("isError", False)
    content = result.get("content", [])
    
    if is_error:
        print("❌ Tool execution failed:")
        for item in content:
            if item.get("type") == "text":
                print(f"   {item.get('text')}")
    else:
        print("✅ Success!")
        for item in content:
            if item.get("type") == "text":
                text = item.get("text", "")
                try:
                    data = json.loads(text)
                    print(json.dumps(data, indent=2))
                    if isinstance(data, list):
                        print(f"\n📊 Total items: {len(data)}")
                except:
                    print(text)

print("\n" + "="*60)
print("✅ Test Complete!")
print("="*60)

proc.terminate()
proc.wait()
