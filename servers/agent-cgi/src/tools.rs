//! # Tool Definition Module
//!
//! Defines the NexusTool trait and tool schemas for the TOOL_DISPATCH worker.
//! Tools are registered and invoked via service binding RPC.
//!
//! ## Tool Categories
//! - **Information**: web_search, memory_recall
//! - **Action**: code_execute, memory_store
//! - **Integration**: External API connectors

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::*;

/// Schema describing a tool's interface (OpenAI-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Unique tool name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for input parameters
    pub input_schema: Value,
}

/// Trait for all NEXUS tools
///
/// Implement this trait to create new tools that can be
/// invoked via the TOOL_DISPATCH worker.
#[async_trait(?Send)]
pub trait NexusTool {
    /// Returns the schema describing this tool's interface
    fn schema(&self) -> ToolSchema;

    /// Executes the tool with given arguments
    ///
    /// # Arguments
    /// * `args` - JSON input matching the tool's input_schema
    /// * `env` - Worker environment for accessing bindings
    ///
    /// # Returns
    /// JSON result or error
    async fn execute(&self, args: Value, env: &Env) -> Result<Value>;
}

// ═══════════════════════════════════════════════════════════════════════════
// BUILT-IN TOOL IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Web Search Tool
/// Searches the web for information using configured search API
pub struct WebSearchTool;

#[async_trait(?Send)]
impl NexusTool for WebSearchTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "web_search".to_string(),
            description: "Search the web for current information on any topic".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "num_results": {
                        "type": "integer",
                        "description": "Number of results to return",
                        "default": 5
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: Value, _env: &Env) -> Result<Value> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("");

        let num_results = args
            .get("num_results")
            .and_then(|n| n.as_u64())
            .unwrap_or(5);

        // TODO: Implement actual search API integration
        // Options: Brave Search, Exa, Tavily, etc.

        Ok(serde_json::json!({
            "query": query,
            "num_results": num_results,
            "results": [],
            "status": "not_implemented",
            "message": "Configure search API in TOOL_DISPATCH worker"
        }))
    }
}

/// Memory Store Tool
/// Persistently stores key-value data in KV or D1
pub struct MemoryStoreTool;

#[async_trait(?Send)]
impl NexusTool for MemoryStoreTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "memory_store".to_string(),
            description: "Store information for later recall. Use for important facts or context.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Unique identifier for this memory"
                    },
                    "value": {
                        "type": "string",
                        "description": "The information to store"
                    },
                    "ttl_seconds": {
                        "type": "integer",
                        "description": "Optional: Time-to-live in seconds"
                    }
                },
                "required": ["key", "value"]
            }),
        }
    }

    async fn execute(&self, args: Value, env: &Env) -> Result<Value> {
        let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("default");
        let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
        let ttl = args.get("ttl_seconds").and_then(|t| t.as_u64());

        if let Ok(kv) = env.kv("CONFIG") {
            let mut put = kv.put(key, value)?;
            if let Some(ttl_secs) = ttl {
                put = put.expiration_ttl(ttl_secs);
            }
            put.execute().await?;

            Ok(serde_json::json!({
                "success": true,
                "key": key,
                "stored": true
            }))
        } else {
            Ok(serde_json::json!({
                "success": false,
                "error": "KV binding not available"
            }))
        }
    }
}

/// Memory Recall Tool
/// Retrieves previously stored information
pub struct MemoryRecallTool;

#[async_trait(?Send)]
impl NexusTool for MemoryRecallTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "memory_recall".to_string(),
            description: "Recall previously stored information by key".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key to recall"
                    }
                },
                "required": ["key"]
            }),
        }
    }

    async fn execute(&self, args: Value, env: &Env) -> Result<Value> {
        let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("default");

        if let Ok(kv) = env.kv("CONFIG") {
            let value = kv.get(key).text().await?;

            Ok(serde_json::json!({
                "key": key,
                "value": value,
                "found": value.is_some()
            }))
        } else {
            Ok(serde_json::json!({
                "key": key,
                "value": null,
                "error": "KV binding not available"
            }))
        }
    }
}

/// Code Execution Tool
/// Executes code snippets in sandboxed environment
pub struct CodeExecuteTool;

#[async_trait(?Send)]
impl NexusTool for CodeExecuteTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "code_execute".to_string(),
            description: "Execute code in a sandboxed environment. Supports Python, JavaScript, and shell commands.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "enum": ["python", "javascript", "shell"],
                        "description": "Programming language"
                    },
                    "code": {
                        "type": "string",
                        "description": "Code to execute"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Execution timeout in milliseconds",
                        "default": 5000
                    }
                },
                "required": ["language", "code"]
            }),
        }
    }

    async fn execute(&self, args: Value, _env: &Env) -> Result<Value> {
        let language = args.get("language").and_then(|l| l.as_str()).unwrap_or("unknown");
        let code = args.get("code").and_then(|c| c.as_str()).unwrap_or("");

        // TODO: Implement sandboxed execution
        // Options: WebContainer, Pyodide (Wasm), Deno subhosting

        Ok(serde_json::json!({
            "language": language,
            "code_length": code.len(),
            "status": "not_implemented",
            "message": "Configure sandboxed execution environment"
        }))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TOOL REGISTRY
// ═══════════════════════════════════════════════════════════════════════════

/// Registry of all available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn NexusTool>>,
}

impl ToolRegistry {
    /// Create a new registry with default tools
    pub fn new() -> Self {
        Self {
            tools: vec![
                Box::new(WebSearchTool),
                Box::new(MemoryStoreTool),
                Box::new(MemoryRecallTool),
                Box::new(CodeExecuteTool),
            ],
        }
    }

    /// Get all tool schemas (for LLM context)
    #[allow(dead_code)]
    pub fn schemas(&self) -> Vec<ToolSchema> {
        self.tools.iter().map(|t| t.schema()).collect()
    }

    /// Find a tool by name
    #[allow(dead_code)]
    pub fn find(&self, name: &str) -> Option<&dyn NexusTool> {
        self.tools
            .iter()
            .find(|t| t.schema().name == name)
            .map(|t| t.as_ref())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
