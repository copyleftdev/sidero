use anyhow::Result;
use serde_json::{json, Value};
use crate::protocol::*;
use crate::semgrep_wrapper::SemgrepWrapper;
use crate::api_client::ApiClient;

pub struct Handler;

impl Handler {
    pub async fn handle_request(req: JsonRpcRequest) -> Result<Value, JsonRpcError> {
        match req.method.as_str() {
            "initialize" => Self::handle_initialize(req.params).await,
            "tools/list" => Self::handle_list_tools().await,
            "tools/call" => Self::handle_call_tool(req.params).await,
            "prompts/list" => Self::handle_list_prompts().await,
            "prompts/get" => Self::handle_get_prompt(req.params).await,
            "resources/list" => Self::handle_list_resources().await,
            "resources/read" => Self::handle_read_resource(req.params).await,
            "notifications/initialized" => Ok(json!(null)), 
             _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
                data: None,
            }),
        }
    }

    async fn handle_initialize(_params: Option<Value>) -> Result<Value, JsonRpcError> {
        let version = SemgrepWrapper::get_version().await.unwrap_or_else(|_| "unknown".to_string());
        
        let result = InitializeResult {
            protocolVersion: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                logging: Some(json!({})),
                tools: Some(json!({"listChanged": false})),
            },
            serverInfo: ServerInfo {
                name: "sidero".to_string(),
                version,
            },
        };

        Ok(serde_json::to_value(result).unwrap())
    }

    // --- Tools ---

    async fn handle_list_tools() -> Result<Value, JsonRpcError> {
        let tools = vec![
            Tool {
                name: "semgrep_scan".to_string(),
                description: Some("Run a Semgrep scan on specific paths".to_string()),
                inputSchema: json!({
                    "type": "object",
                    "properties": {
                        "paths": { "type": "array", "items": { "type": "string" }, "description": "List of file paths to scan" },
                        "config": { "type": "string", "description": "Rule configuration" }
                    },
                    "required": ["paths"]
                }),
            },
            Tool {
                name: "semgrep_scan_with_custom_rule".to_string(),
                description: Some("Run a scan with a custom ad-hoc rule".to_string()),
                inputSchema: json!({
                    "type": "object",
                    "properties": {
                        "rule": { "type": "string", "description": "YAML rule content" },
                        "code_files": { "type": "array", "items": { "type": "string" }, "description": "Files to scan" }
                    },
                    "required": ["rule", "code_files"]
                }),
            },
            Tool {
                name: "get_abstract_syntax_tree".to_string(),
                description: Some("Get the AST of a code snippet".to_string()),
                inputSchema: json!({
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Code content" },
                        "language": { "type": "string", "description": "Language of the code" }
                    },
                    "required": ["code", "language"]
                }),
            },
             Tool {
                name: "semgrep_findings".to_string(),
                description: Some("Fetch Semgrep findings".to_string()),
                 inputSchema: json!({
                    "type": "object",
                    "properties": {
                        "issue_type": { "type": "string" },
                        "status": { "type": "string" },
                         "repos": { "type": "array", "items": { "type": "string" } },
                         "severities": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "get_version".to_string(),
                description: Some("Get Semgrep version".to_string()),
                inputSchema: json!({ "type": "object", "properties": {} }),
            },
             Tool {
                name: "supported_languages".to_string(),
                description: Some("List supported languages".to_string()),
                inputSchema: json!({ "type": "object", "properties": {} }),
            },
        ];

        Ok(serde_json::to_value(ListToolsResult { tools }).unwrap())
    }

    async fn handle_call_tool(params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: CallToolParams = serde_json::from_value(params.unwrap_or(json!({}))).map_err(|e| JsonRpcError {
                code: -32602, message: format!("Invalid params: {}", e), data: None,
        })?;

        match params.name.as_str() {
            "get_version" => {
                let version = SemgrepWrapper::get_version().await.map_err(internal_error)?;
                Ok(json!(CallToolResult { content: vec![Content::Text { text: version }], isError: None }))
            }
             "supported_languages" => {
                let langs = SemgrepWrapper::get_supported_languages().await.map_err(internal_error)?;
                Ok(json!(CallToolResult { content: vec![Content::Text { text: langs.join(", ") }], isError: None }))
            }
            "semgrep_scan" => {
                let args = params.arguments.unwrap_or(json!({}));
                let paths: Vec<String> = serde_json::from_value(args.get("paths").unwrap_or(&json!([])).clone()).map_err(|_| JsonRpcError {
                     code: -32602, message: "Invalid paths".to_string(), data: None
                })?;
                let config = args.get("config").and_then(|v| v.as_str()).map(|s| s.to_string());
                let result = SemgrepWrapper::scan(config, paths).await.map_err(internal_error)?;
                Ok(json!(CallToolResult { content: vec![Content::Text { text: serde_json::to_string_pretty(&result).unwrap() }], isError: None }))
            }
            "semgrep_scan_with_custom_rule" => {
                 let args = params.arguments.unwrap_or(json!({}));
                 let rule = args.get("rule").and_then(|v| v.as_str()).ok_or(JsonRpcError { code: -32602, message: "Missing rule".to_string(), data: None })?.to_string();
                 let files: Vec<String> = serde_json::from_value(args.get("code_files").unwrap_or(&json!([])).clone()).map_err(|_| JsonRpcError { code: -32602, message: "Invalid code_files".to_string(), data: None })?;
                 
                 let result = SemgrepWrapper::scan_with_custom_rule(rule, files).await.map_err(internal_error)?;
                 Ok(json!(CallToolResult { content: vec![Content::Text { text: serde_json::to_string_pretty(&result).unwrap() }], isError: None }))
            }
            "get_abstract_syntax_tree" => {
                let args = params.arguments.unwrap_or(json!({}));
                let code = args.get("code").and_then(|v| v.as_str()).ok_or(JsonRpcError { code: -32602, message: "Missing code".to_string(), data: None })?.to_string();
                let lang = args.get("language").and_then(|v| v.as_str()).ok_or(JsonRpcError { code: -32602, message: "Missing language".to_string(), data: None })?.to_string();
                
                let result = SemgrepWrapper::dump_ast(code, lang).await.map_err(internal_error)?;
                Ok(json!(CallToolResult { content: vec![Content::Text { text: serde_json::to_string_pretty(&result).unwrap() }], isError: None }))

            }
            "semgrep_findings" => {
                let token = std::env::var("SEMGREP_APP_TOKEN").map_err(|_| JsonRpcError { code: -32603, message: "SEMGREP_APP_TOKEN not set".to_string(), data: None })?;
                let args = params.arguments.unwrap_or(json!({}));
                let mut q = serde_json::Map::new();
                if let Some(obj) = args.as_object() {
                    for (k, v) in obj {
                        if k == "repos" && v.is_array() {
                            let s = v.as_array().unwrap().iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(",");
                            q.insert(k.clone(), Value::String(s));
                        } else if !v.is_null() {
                            q.insert(k.clone(), v.clone());
                        }
                    }
                }
                let res = ApiClient::get_findings(&token, q).await.map_err(internal_error)?;
                 Ok(json!(CallToolResult { content: vec![Content::Text { text: serde_json::to_string_pretty(&res).unwrap() }], isError: None }))
            }
            _ => Err(JsonRpcError { code: -32601, message: format!("Tool not found: {}", params.name), data: None }),
        }
    }

    // --- Prompts ---

    async fn handle_list_prompts() -> Result<Value, JsonRpcError> {
        let prompts = vec![
            Prompt {
                name: "write_custom_semgrep_rule".to_string(),
                description: Some("Helper to write a custom Semgrep rule".to_string()),
                arguments: Some(vec![
                    PromptArgument { name: "code".to_string(), description: Some("Code snippet".to_string()), required: Some(true) },
                    PromptArgument { name: "language".to_string(), description: Some("Language".to_string()), required: Some(true) },
                ])
            }
        ];
        Ok(serde_json::to_value(ListPromptsResult { prompts }).unwrap())
    }

    async fn handle_get_prompt(params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: GetPromptParams = serde_json::from_value(params.unwrap_or(json!({}))).map_err(|e| JsonRpcError {
             code: -32602, message: format!("Invalid params: {}", e), data: None
        })?;

        if params.name == "write_custom_semgrep_rule" {
            let args = params.arguments.unwrap_or_default();
            let code = args.get("code").unwrap_or(&"".to_string()).clone();
            let lang = args.get("language").unwrap_or(&"".to_string()).clone();

            let prompt_text = format!(
                "You are an expert at writing Semgrep rules.\n\nCode to analyze:\n```{}\n{}\n```\n\nLanguage: {}\n\nCreate a Semgrep rule to detect issues in this code.",
                lang, code, lang
            );

            Ok(serde_json::to_value(GetPromptResult {
                description: Some("Write custom rule".to_string()),
                messages: vec![
                    PromptMessage {
                        role: "user".to_string(),
                        content: Content::Text { text: prompt_text }
                    }
                ]
            }).unwrap())
        } else {
             Err(JsonRpcError { code: -32601, message: "Prompt not found".to_string(), data: None })
        }
    }

    // --- Resources ---

    async fn handle_list_resources() -> Result<Value, JsonRpcError> {
        let resources = vec![
            Resource {
                uri: "semgrep://rule/schema".to_string(),
                name: "Semgrep Rule Schema".to_string(),
                description: Some("JSON Schema for Semgrep Rules".to_string()),
                mimeType: Some("application/json".to_string()),
            }
        ];
        Ok(serde_json::to_value(ListResourcesResult { resources }).unwrap())
    }

    async fn handle_read_resource(params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: ReadResourceParams = serde_json::from_value(params.unwrap_or(json!({}))).map_err(|e| JsonRpcError {
             code: -32602, message: format!("Invalid params: {}", e), data: None
        })?;

        let uri = params.uri.as_str();
        let content = if uri == "semgrep://rule/schema" {
             ApiClient::fetch_url("https://raw.githubusercontent.com/semgrep/semgrep-interfaces/refs/heads/main/rule_schema_v1.yaml").await.map_err(internal_error)?
        } else if uri.starts_with("semgrep://rule/") && uri.ends_with("/yaml") {
            // Extract rule ID
             // semgrep://rule/{id}/yaml
             let parts: Vec<&str> = uri.split('/').collect();
             if parts.len() >= 4 {
                 let rule_id = parts[2];
                 ApiClient::fetch_url(&format!("https://semgrep.dev/c/r/{}", rule_id)).await.map_err(internal_error)?
             } else {
                 return Err(JsonRpcError { code: -32602, message: "Invalid resource URI".to_string(), data: None });
             }
        } else {
             return Err(JsonRpcError { code: -32602, message: "Resource not found".to_string(), data: None });
        };

        Ok(serde_json::to_value(ReadResourceResult {
            contents: vec![
                ResourceContent {
                    uri: params.uri,
                    mimeType: Some("text/plain".to_string()),
                    text: content
                }
            ]
        }).unwrap())
    }
}

fn internal_error<E: std::fmt::Display>(e: E) -> JsonRpcError {
    JsonRpcError {
        code: -32603,
        message: e.to_string(),
        data: None,
    }
}
