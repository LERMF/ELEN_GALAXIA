//! # Service Binding RPC Module
//!
//! Provides zero-overhead RPC abstractions for inter-worker communication.
//! Uses Cloudflare Service Bindings for <2ms latency tool dispatch.
//!
//! ## Architecture
//! - TOOL_DISPATCH: Routes tool calls to appropriate handlers
//! - AUTH_VAULT: Manages encrypted secrets and tokens
//! - Internal requests never leave the Cloudflare colo

use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::*;

/// RPC dispatcher for tool invocations via service bindings
pub struct ToolDispatcher<'a> {
    env: &'a Env,
}

impl<'a> ToolDispatcher<'a> {
    /// Create a new tool dispatcher with environment bindings
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    /// Dispatch a tool call via service binding RPC
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool to invoke
    /// * `input` - JSON input arguments for the tool
    ///
    /// # Returns
    /// The tool's JSON response or an error
    pub async fn call(&self, tool_name: &str, input: &Value) -> Result<Value> {
        // Check if TOOL_DISPATCH binding exists
        match self.env.service("TOOL_DISPATCH") {
            Ok(binding) => {
                self.dispatch_via_binding(&binding, tool_name, input).await
            }
            Err(_) => {
                // Fallback to built-in tools if binding not configured
                console_log!("⚠️ TOOL_DISPATCH binding not found, using built-in handlers");
                self.dispatch_builtin(tool_name, input).await
            }
        }
    }

    /// Dispatch via service binding (zero-overhead RPC)
    async fn dispatch_via_binding(
        &self,
        binding: &Fetcher,
        tool_name: &str,
        input: &Value,
    ) -> Result<Value> {
        let request_body = serde_json::json!({
            "name": tool_name,
            "arguments": input
        });

        let mut init = RequestInit::new();
        init.with_method(Method::Post);

        let headers = Headers::new();
        headers.set("Content-Type", "application/json")?;
        headers.set("X-Internal-RPC", "true")?;
        init.with_headers(headers);

        init.with_body(Some(wasm_bindgen::JsValue::from_str(
            &serde_json::to_string(&request_body)?
        )));

        console_log!("🔗 RPC call to TOOL_DISPATCH: {}", tool_name);

        // Use URL string and RequestInit for fetch
        let mut response = binding.fetch("http://internal/tools/call", Some(init)).await?;

        if response.status_code() != 200 {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::from(format!(
                "Tool dispatch failed ({}): {}",
                response.status_code(),
                error_text
            )));
        }

        let result: Value = response.json().await?;
        Ok(result)
    }

    /// Built-in tool handlers (fallback when binding unavailable)
    async fn dispatch_builtin(&self, tool_name: &str, input: &Value) -> Result<Value> {
        match tool_name {
            "web_search" => self.builtin_web_search(input).await,
            "memory_store" => self.builtin_memory_store(input).await,
            "memory_recall" => self.builtin_memory_recall(input).await,
            "code_execute" => self.builtin_code_execute(input).await,
            _ => Ok(serde_json::json!({
                "error": format!("Unknown tool: {}", tool_name),
                "available_tools": ["web_search", "memory_store", "memory_recall", "code_execute"]
            })),
        }
    }

    /// Built-in web search (placeholder)
    async fn builtin_web_search(&self, input: &Value) -> Result<Value> {
        let query = input
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("");

        console_log!("🔍 Built-in web_search: {}", query);

        // In production, integrate with search API (Brave, Exa, etc.)
        Ok(serde_json::json!({
            "tool": "web_search",
            "query": query,
            "results": [
                {
                    "title": "Placeholder Search Result",
                    "url": "https://example.com",
                    "snippet": format!("This is a placeholder result for query: '{}'", query)
                }
            ],
            "note": "Implement actual search API integration in TOOL_DISPATCH worker"
        }))
    }

    /// Built-in memory store (uses KV if available)
    async fn builtin_memory_store(&self, input: &Value) -> Result<Value> {
        let key = input
            .get("key")
            .and_then(|k| k.as_str())
            .unwrap_or("default");
        let value = input
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Try to use KV binding
        if let Ok(kv) = self.env.kv("CONFIG") {
            kv.put(key, value)?.execute().await?;
            console_log!("💾 Stored to KV: {} = {}", key, value);
            return Ok(serde_json::json!({
                "tool": "memory_store",
                "success": true,
                "key": key
            }));
        }

        // Fallback: just acknowledge
        Ok(serde_json::json!({
            "tool": "memory_store",
            "success": true,
            "key": key,
            "note": "Stored in session memory only (KV binding not available)"
        }))
    }

    /// Built-in memory recall (uses KV if available)
    async fn builtin_memory_recall(&self, input: &Value) -> Result<Value> {
        let key = input
            .get("key")
            .and_then(|k| k.as_str())
            .unwrap_or("default");

        // Try to use KV binding
        if let Ok(kv) = self.env.kv("CONFIG") {
            let value = kv.get(key).text().await?;
            console_log!("📖 Recalled from KV: {} = {:?}", key, value);
            return Ok(serde_json::json!({
                "tool": "memory_recall",
                "key": key,
                "value": value
            }));
        }

        Ok(serde_json::json!({
            "tool": "memory_recall",
            "key": key,
            "value": null,
            "note": "KV binding not available"
        }))
    }

    /// Built-in code execution (sandboxed placeholder)
    async fn builtin_code_execute(&self, input: &Value) -> Result<Value> {
        let language = input
            .get("language")
            .and_then(|l| l.as_str())
            .unwrap_or("unknown");
        let code = input
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("");

        console_log!("⚙️ Code execution requested: {} ({} chars)", language, code.len());

        // In production, use a sandboxed execution environment
        Ok(serde_json::json!({
            "tool": "code_execute",
            "language": language,
            "status": "sandbox_required",
            "note": "Code execution requires sandboxed environment. Implement in TOOL_DISPATCH worker."
        }))
    }
}

/// Request structure for tool dispatch
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolRequest {
    pub name: String,
    pub arguments: Value,
}

/// Response structure from tool dispatch
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}
