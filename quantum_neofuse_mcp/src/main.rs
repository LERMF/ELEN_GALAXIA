mod quantum;
mod agent;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use quantum::QuantumBranch;
use agent::{AgentContext, AgentGraph};

#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

fn main() -> Result<()> {
    // Quantum Initiation
    eprintln!("🚀 RUST_MCP_QuantumNeoFuse Initializing...");
    
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        
        // Parse request
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                continue;
            }
        };

        // If it's a notification (no id), handle it but don't respond
        if req.id.is_none() {
            eprintln!("Received notification: {}", req.method);
            continue;
        }

        // Router
        let result = match req.method.as_str() {
            "initialize" => handle_initialize(),
            "tools/list" => handle_list_tools(),
            "tools/call" => handle_call_tool(req.params),
            method => Err(anyhow::anyhow!("Method not found: {}", method)),
        };

        // Respond
        let (res_val, err_val) = match result {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(serde_json::json!({
                "code": -32603,
                "message": e.to_string()
            })))
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: res_val,
            error: err_val,
            id: req.id,
        };
        
        println!("{}", serde_json::to_string(&response)?);
        io::stdout().flush()?;
    }

    Ok(())
}

fn handle_initialize() -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "quantum_neofuse_mcp",
            "version": "0.1.0"
        }
    }))
}

fn handle_list_tools() -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "tools": [
            {
                "name": "quantum_sim",
                "description": "Run a Ψ-Branching Quantum Simulation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "branches": { "type": "integer" }
                    }
                }
            },
            {
                "name": "agent_react_step",
                "description": "Execute a ReAct Agent Step via GraphFlow",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }
            }
        ]
    }))
}

fn handle_call_tool(params: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let params = params.unwrap_or(serde_json::Value::Null);
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    match name {
        "quantum_sim" => {
            let branches = args.get("branches").and_then(|b| b.as_u64()).unwrap_or(2) as usize;
            let result = QuantumBranch::psi_branch(branches);
            Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string(&result)? }] }))
        },
        "agent_react_step" => {
            let input = args.get("input").and_then(|s| s.as_str()).unwrap_or("Thinking...");
            let mut ctx = AgentContext::new();
            let result = AgentGraph::step(&mut ctx, input);
            Ok(serde_json::json!({ "content": [{ "type": "text", "text": result }] }))
        },
        _ => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Tool not found" }] }))
    }
}
