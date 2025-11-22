use anyhow::{Context, Result};
use openapiv3::{OpenAPI, Operation, Parameter, ParameterSchemaOrContent, ReferenceOr, SchemaKind, Type};
use reqwest::Method;
use rmcp::{
    model::*,
    service::RequestContext,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    transport::stdio,
};
use serde_json::{json, Map, Value};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Main application state containing configuration and OpenAPI spec
#[derive(Clone)]
struct OpenApiServer {
    base_url: String,
    #[allow(dead_code)]
    doc_url: String,
    openapi_spec: Arc<OpenAPI>,
    http_client: reqwest::Client,
    tools: Arc<Vec<Tool>>,
}

impl OpenApiServer {
    /// Fetch and parse OpenAPI specification from the doc_url
    async fn new(base_url: String, doc_url: String) -> Result<Self> {
        info!("🔍 Fetching OpenAPI spec from: {}", doc_url);

        let http_client = reqwest::Client::new();
        let spec_text = http_client
            .get(&doc_url)
            .send()
            .await
            .context("Failed to fetch OpenAPI spec")?
            .text()
            .await
            .context("Failed to read spec body")?;

        let openapi_spec: OpenAPI = serde_json::from_str(&spec_text)
            .context("Failed to parse OpenAPI spec as JSON")?;

        info!(
            "✅ Successfully loaded OpenAPI spec: {} v{}",
            openapi_spec.info.title,
            openapi_spec.info.version
        );

        let tools = Self::generate_tools_from_spec(&openapi_spec);
        info!("🛠️  Generated {} tools from OpenAPI spec", tools.len());

        Ok(Self {
            base_url,
            doc_url,
            openapi_spec: Arc::new(openapi_spec),
            http_client,
            tools: Arc::new(tools),
        })
    }

    /// Generate MCP tools from OpenAPI operations
    fn generate_tools_from_spec(spec: &OpenAPI) -> Vec<Tool> {
        let mut tools = Vec::new();

        for (path, path_item) in &spec.paths.paths {
            let path_item = match path_item {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { .. } => continue,
            };

            // Process each HTTP method
            Self::process_operation_static(spec, path, &path_item.get, "GET", &mut tools);
            Self::process_operation_static(spec, path, &path_item.post, "POST", &mut tools);
            Self::process_operation_static(spec, path, &path_item.put, "PUT", &mut tools);
            Self::process_operation_static(spec, path, &path_item.delete, "DELETE", &mut tools);
            Self::process_operation_static(spec, path, &path_item.patch, "PATCH", &mut tools);
        }

        tools
    }

    /// Process a single operation and add it as a tool
    fn process_operation_static(
        spec: &OpenAPI,
        path: &str,
        operation: &Option<Operation>,
        method: &str,
        tools: &mut Vec<Tool>,
    ) {
        if let Some(op) = operation {
            let tool_name = format!(
                "{}_{}",
                method.to_lowercase(),
                path.replace('/', "_")
                    .replace('{', "")
                    .replace('}', "")
                    .trim_matches('_')
            );

            let description = op
                .summary
                .as_ref()
                .or(op.description.as_ref())
                .cloned()
                .unwrap_or_else(|| format!("{} {}", method, path));

            // Build input schema from parameters
            let mut properties = Map::new();
            let mut required = Vec::new();

            for param in &op.parameters {
                if let ReferenceOr::Item(param) = param {
                    let (param_data, is_path_param) = match param {
                        Parameter::Query { parameter_data, .. } => (parameter_data, false),
                        Parameter::Path { parameter_data, .. } => (parameter_data, true),
                        Parameter::Header { parameter_data, .. } => (parameter_data, false),
                        Parameter::Cookie { parameter_data, .. } => (parameter_data, false),
                    };

                    // Add to required list if it's a path param or explicitly marked as required
                    if is_path_param || param_data.required {
                        if !required.contains(&param_data.name) {
                            required.push(param_data.name.clone());
                        }
                    }

                    // Extract schema from parameter
                    let schema = match &param_data.format {
                        ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
                            ReferenceOr::Item(_schema) => {
                                let mut prop = Map::new();
                                prop.insert("type".to_string(), json!("string"));

                                if let Some(desc) = &param_data.description {
                                    prop.insert("description".to_string(), json!(desc));
                                }

                                json!(prop)
                            }
                            ReferenceOr::Reference { .. } => json!({"type": "string"}),
                        },
                        ParameterSchemaOrContent::Content(_) => json!({"type": "string"}),
                    };

                    properties.insert(param_data.name.clone(), schema);
                }
            }

            // Flatten request body properties directly into parameters
            if let Some(request_body) = &op.request_body {
                if let ReferenceOr::Item(body) = request_body {
                    // Try to extract schema from application/json content
                    if let Some(content) = body.content.get("application/json") {
                        if let Some(media_schema) = &content.schema {
                            // Resolve schema (handle both inline and references)
                            let resolved_schema = match media_schema {
                                ReferenceOr::Item(schema) => Some(schema),
                                ReferenceOr::Reference { reference } => {
                                    // Extract schema name from reference like "#/components/schemas/Pet"
                                    if let Some(schema_name) = reference.strip_prefix("#/components/schemas/") {
                                        spec.components.as_ref()
                                            .and_then(|c| c.schemas.get(schema_name))
                                            .and_then(|s| match s {
                                                ReferenceOr::Item(schema) => Some(schema),
                                                _ => None,
                                            })
                                    } else {
                                        None
                                    }
                                }
                            };
                            
                            if let Some(schema) = resolved_schema {
                                // Extract properties from the schema
                                if let SchemaKind::Type(Type::Object(obj_type)) = &schema.schema_kind {
                                    for (prop_name, prop_schema_ref) in &obj_type.properties {
                                        // Convert the property schema to JSON
                                        let prop_json = match prop_schema_ref {
                                            ReferenceOr::Item(prop_schema) => {
                                                let mut prop_obj = Map::new();
                                                
                                                // Determine the type
                                                match &prop_schema.schema_kind {
                                                    SchemaKind::Type(Type::String(_)) => {
                                                        prop_obj.insert("type".to_string(), json!("string"));
                                                    }
                                                    SchemaKind::Type(Type::Number(_)) => {
                                                        prop_obj.insert("type".to_string(), json!("number"));
                                                    }
                                                    SchemaKind::Type(Type::Integer(_)) => {
                                                        prop_obj.insert("type".to_string(), json!("integer"));
                                                    }
                                                    SchemaKind::Type(Type::Boolean(_)) => {
                                                        prop_obj.insert("type".to_string(), json!("boolean"));
                                                    }
                                                    SchemaKind::Type(Type::Array(_)) => {
                                                        prop_obj.insert("type".to_string(), json!("array"));
                                                    }
                                                    SchemaKind::Type(Type::Object(_)) => {
                                                        prop_obj.insert("type".to_string(), json!("object"));
                                                    }
                                                    _ => {
                                                        prop_obj.insert("type".to_string(), json!("string"));
                                                    }
                                                }
                                                
                                                // Add description if available
                                                if let Some(desc) = &prop_schema.schema_data.description {
                                                    prop_obj.insert("description".to_string(), json!(desc));
                                                }
                                                
                                                json!(prop_obj)
                                            }
                                            ReferenceOr::Reference { .. } => {
                                                json!({"type": "string"})
                                            }
                                        };
                                        
                                        properties.insert(prop_name.clone(), prop_json);
                                    }
                                    
                                    // Add required properties from the schema
                                    for req_prop in &obj_type.required {
                                        if !required.contains(req_prop) {
                                            required.push(req_prop.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let input_schema = json!({
                "type": "object",
                "properties": properties,
                "required": required
            });

            tools.push(Tool {
                name: Cow::Owned(tool_name.clone()),
                description: Some(Cow::Owned(description)),
                input_schema: Arc::new(input_schema.as_object().unwrap().clone()),
                annotations: None,
                icons: Some(Vec::new()),
                meta: None,
                title: None,
                output_schema: None,
            });
        }
    }

    /// Execute an API call based on tool invocation
    async fn execute_tool(&self, tool_name: &str, arguments: Value) -> Result<String> {
        info!("🚀 Executing tool: {} with args: {}", tool_name, arguments);

        // Parse tool name to extract method and path
        let parts: Vec<&str> = tool_name.splitn(2, '_').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid tool name format: {}", tool_name);
        }

        let method = parts[0].to_uppercase();
        let path_part = parts[1]; // Keep underscores for matching

        // Find matching path in OpenAPI spec
        let (path, operation) = self.find_operation(&method, path_part)?;

        // Build the request URL
        let mut url = format!("{}{}", self.base_url.trim_end_matches('/'), path);

        // Extract arguments
        let args_obj = arguments
            .as_object()
            .context("Arguments must be an object")?;

        // Replace path parameters
        for (key, value) in args_obj {
            if path.contains(&format!("{{{}}}", key)) {
                url = url.replace(
                    &format!("{{{}}}", key),
                    &value.to_string().trim_matches('"'),
                );
            }
        }

        // Collect all parameter names that are path or query params
        let mut used_param_names = std::collections::HashSet::new();
        
        // Build query parameters
        let mut query_params = Vec::new();
        for param in &operation.parameters {
            if let ReferenceOr::Item(param_item) = param {
                let param_data = match param_item {
                    Parameter::Query { parameter_data, .. } => {
                        used_param_names.insert(parameter_data.name.clone());
                        Some(parameter_data)
                    }
                    Parameter::Path { parameter_data, .. } => {
                        used_param_names.insert(parameter_data.name.clone());
                        None
                    }
                    Parameter::Header { parameter_data, .. } => {
                        used_param_names.insert(parameter_data.name.clone());
                        None
                    }
                    Parameter::Cookie { parameter_data, .. } => {
                        used_param_names.insert(parameter_data.name.clone());
                        None
                    }
                };
                
                if let Some(param_data) = param_data {
                    if let Some(value) = args_obj.get(&param_data.name) {
                        query_params.push((
                            param_data.name.clone(),
                            value.to_string().trim_matches('"').to_string(),
                        ));
                    }
                }
            }
        }

        // Build and execute request
        let method_enum = Method::from_bytes(method.as_bytes())?;
        let mut request = self.http_client.request(method_enum, &url);

        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        // Build body from remaining parameters (those not used in path/query/header/cookie)
        let body_params: Map<String, Value> = args_obj
            .iter()
            .filter(|(key, _)| !used_param_names.contains(key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Add body if there are any body parameters
        if !body_params.is_empty() {
            request = request.json(&body_params);
        }

        info!("📡 Sending request to: {}", url);
        let response = request
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            info!("✅ Request succeeded with status {}", status);
            Ok(response_text)
        } else {
            error!("❌ Request failed with status {}", status);
            anyhow::bail!(
                "API request failed with status {}: {}",
                status,
                response_text
            )
        }
    }

    /// Find the operation in OpenAPI spec matching method and path pattern
    fn find_operation(&self, method: &str, path_pattern: &str) -> Result<(&str, &Operation)> {
        for (path, path_item) in &self.openapi_spec.paths.paths {
            // Check if path matches (accounting for path parameters)
            let path_normalized = path
                .replace('{', "")
                .replace('}', "")
                .replace('/', "_")
                .trim_matches('_')
                .to_string();

            if path_pattern == path_normalized {
                let path_item = match path_item {
                    ReferenceOr::Item(item) => item,
                    ReferenceOr::Reference { .. } => continue,
                };

                let operation = match method {
                    "GET" => path_item.get.as_ref(),
                    "POST" => path_item.post.as_ref(),
                    "PUT" => path_item.put.as_ref(),
                    "DELETE" => path_item.delete.as_ref(),
                    "PATCH" => path_item.patch.as_ref(),
                    _ => None,
                };

                if let Some(op) = operation {
                    return Ok((path.as_str(), op));
                }
            }
        }

        anyhow::bail!("Operation not found for {} {}", method, path_pattern)
    }
}

impl ServerHandler for OpenApiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "mcp-openapi-transformer".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                icons: Some(Vec::new()),
                website_url: None,
            },
            instructions: Some(format!(
                "This server exposes REST API endpoints from {} as MCP tools. Base URL: {}. {} tools available.",
                self.openapi_spec.info.title,
                self.base_url,
                self.tools.len()
            )),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("🔧 Client connected, initializing...");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        info!("📋 Client requested tools list");
        let tools_count = self.tools.len();
        info!("📊 Returning {} tools", tools_count);
        
        let result = ListToolsResult {
            tools: self.tools.as_ref().clone(),
            next_cursor: None,
        };
        
        info!("✅ Tools list prepared successfully");
        Ok(result)
    }

    async fn call_tool(
        &self,
        CallToolRequestParam { name, arguments }: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let args = match arguments {
            Some(args) => Value::Object(args),
            None => json!({}),
        };

        match self.execute_tool(&name, args).await {
            Ok(result) => Ok(CallToolResult {
                content: vec![Content::text(result)],
                is_error: Some(false),
                meta: None,
                structured_content: None,
            }),
            Err(e) => {
                error!("❌ Tool execution failed: {}", e);
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Error: {}", e))],
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                })
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("🚀 Starting MCP OpenAPI Transformer");

    // Get configuration from environment variables
    let base_url =
        env::var("BASE_URL").context("BASE_URL environment variable is required")?;
    let doc_url = env::var("DOC_URL").context("DOC_URL environment variable is required")?;

    info!("📍 Base URL: {}", base_url);
    info!("📄 OpenAPI Doc URL: {}", doc_url);

    // Initialize app state
    let server = OpenApiServer::new(base_url, doc_url).await?;

    info!("✨ MCP Server ready. Waiting for requests...");

    // Start the server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("❌ Server error: {:?}", e);
    })?;

    service.waiting().await?;

    info!("👋 Server shutting down");
    Ok(())
}
