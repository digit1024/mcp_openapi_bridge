#!/usr/bin/env bash

# Example script to run the MCP OpenAPI Transformer with the Petstore API

export BASE_URL="https://petstore3.swagger.io/api/v3"
export DOC_URL="https://petstore3.swagger.io/api/v3/openapi.json"
export RUST_LOG="info"

echo "🚀 Starting MCP OpenAPI Transformer"
echo "📍 Base URL: $BASE_URL"
echo "📄 OpenAPI Doc: $DOC_URL"
echo ""

unset ARGV0
cargo run --release
