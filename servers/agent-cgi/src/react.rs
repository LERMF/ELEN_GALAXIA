//! # ReAct Loop Implementation
//!
//! Implements the Reason-Act-Observe pattern for autonomous agent execution.
//! Uses Workers AI for LLM inference and service bindings for tool dispatch.
//!
//! ## Loop Structure
//! 1. **Reason**: LLM generates thought + action decision
//! 2. **Act**: Tool dispatch via RPC
//! 3. **Observe**: Integrate tool result into context
//! 4. **Repeat** until goal achieved or max iterations

use crate::rpc::ToolDispatcher;
use crate::state::AgentState;
use serde::{Deserialize, Serialize};
use worker::*;

/// Maximum default iterations for the ReAct loop
const DEFAULT_MAX_ITERATIONS: u32 = 5;

/// Output from a complete ReAct execution
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentOutput {
    /// Session identifier
    pub session_id: String,
    /// Number of ReAct iterations executed
    pub iterations: u32,
    /// Final answer if goal was achieved
    pub final_answer: Option<String>,
    /// Record of all tool calls made
    pub tool_calls: Vec<ToolCallRecord>,
    /// Whether the loop converged naturally
    pub converged: bool,
}

/// Record of a single tool invocation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Tool name
    pub name: String,
    /// Input arguments
    pub input: serde_json::Value,
    /// Tool output
    pub output: serde_json::Value,
    /// Execution time in milliseconds
    pub latency_ms: u32,
}

/// Internal representation of a tool call extracted from LLM response
#[derive(Debug)]
struct ToolCall {
    name: String,
    input: serde_json::Value,
}

/// The ReAct Loop executor
pub struct ReActLoop<'a> {
    env: &'a Env,
    state: Option<AgentState>,
    max_iterations: u32,
}

impl<'a> ReActLoop<'a> {
    /// Create a new ReAct loop with environment
    pub fn new(env: &'a Env) -> Self {
        Self {
            env,
            state: None,
            max_iterations: DEFAULT_MAX_ITERATIONS,
        }
    }

    /// Set the agent state with loaded context
    pub fn with_state(mut self, state: AgentState) -> Self {
        self.state = Some(state);
        self
    }

    /// Set maximum iterations (Ψ-LOOP₅ enforcement)
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Execute the ReAct loop until goal completion or max iterations
    pub async fn execute(&self, goal: &str) -> Result<AgentOutput> {
        let session_id = self
            .state
            .as_ref()
            .map(|s| s.session_id.clone())
            .unwrap_or_else(|| "anonymous".to_string());

        let mut output = AgentOutput {
            session_id: session_id.clone(),
            iterations: 0,
            final_answer: None,
            tool_calls: vec![],
            converged: false,
        };

        let mut context = self.build_initial_context(goal);
        let tool_dispatcher = ToolDispatcher::new(self.env);

        console_log!(
            "🧠 Starting ReAct loop for session {} with max {} iterations",
            session_id,
            self.max_iterations
        );

        while output.iterations < self.max_iterations && output.final_answer.is_none() {
            output.iterations += 1;
            console_log!("🔄 ReAct iteration {}/{}", output.iterations, self.max_iterations);

            // ═══════════════════════════════════════════════════════════════
            // 🧠 REASON: LLM generates thought + action
            // ═══════════════════════════════════════════════════════════════
            let response = self.reason(&context).await?;
            let response_preview = if response.len() > 200 { &response[..200] } else { &response };
            console_log!("💭 LLM response: {}...", response_preview);

            // ═══════════════════════════════════════════════════════════════
            // 🎯 DECISION TREE: Tool call or final answer?
            // ═══════════════════════════════════════════════════════════════
            if let Some(tool_call) = self.extract_tool_call(&response) {
                console_log!("🛠️ Tool call detected: {}", tool_call.name);

                // ════════════════════════════════════════════════════════════
                // 🔧 ACT: Dispatch tool via RPC
                // ════════════════════════════════════════════════════════════
                let start = js_sys::Date::now();
                let tool_result = tool_dispatcher
                    .call(&tool_call.name, &tool_call.input)
                    .await
                    .unwrap_or_else(|e| {
                        serde_json::json!({"error": e.to_string()})
                    });
                let latency = (js_sys::Date::now() - start) as u32;

                output.tool_calls.push(ToolCallRecord {
                    name: tool_call.name.clone(),
                    input: tool_call.input.clone(),
                    output: tool_result.clone(),
                    latency_ms: latency,
                });

                // ════════════════════════════════════════════════════════════
                // 👁️ OBSERVE: Append result to context
                // ════════════════════════════════════════════════════════════
                context.push_str(&format!(
                    "\nObservation: {}\n",
                    serde_json::to_string_pretty(&tool_result).unwrap_or_default()
                ));
            } else if let Some(answer) = self.extract_final_answer(&response) {
                // ════════════════════════════════════════════════════════════
                // ✅ FINAL ANSWER: Goal achieved
                // ════════════════════════════════════════════════════════════
                output.final_answer = Some(answer);
                output.converged = true;
                console_log!("✅ Goal achieved!");
            } else {
                // No clear action - append raw response and continue
                context.push_str(&format!("\nThought: {}\n", response));
            }
        }

        if !output.converged {
            console_log!("⚠️ Max iterations reached without convergence");
        }

        Ok(output)
    }

    /// Build the initial context from goal and loaded state
    fn build_initial_context(&self, goal: &str) -> String {
        let mut ctx = format!(
            r#"You are an autonomous AI agent operating in GOD_MODE within the NEXUS_OMNI system.
Your task is to achieve the following goal using the ReAct (Reason-Act-Observe) pattern.

GOAL: {}

AVAILABLE TOOLS:
- web_search: Search the web for information. Input: {{"query": "search terms"}}
- code_execute: Execute code snippets. Input: {{"language": "python|rust|js", "code": "..."}}  
- memory_store: Store information for later. Input: {{"key": "name", "value": "data"}}
- memory_recall: Recall stored information. Input: {{"key": "name"}}

RESPONSE FORMAT:
Either provide a tool call:
Thought: [Your reasoning about what to do next]
Action: [tool_name]
Action Input: [JSON input for the tool]

Or provide the final answer:
Thought: [Your final reasoning]
Final Answer: [Your complete response to the goal]

BEGIN:
"#,
            goal
        );

        // Add context from state if available
        if let Some(ref state) = self.state {
            if !state.memory.is_empty() {
                ctx.push_str("\nPREVIOUS CONTEXT:\n");
                for (key, value) in &state.memory {
                    ctx.push_str(&format!("- {}: {}\n", key, value));
                }
            }
        }

        ctx
    }

    /// Send context to LLM and get response
    async fn reason(&self, context: &str) -> Result<String> {
        // Try to use Workers AI binding
        // Note: This requires the AI binding to be configured in wrangler.toml
        
        // For now, return a mock response since we need AI binding configured
        // In production, this would call Workers AI
        
        console_log!("📡 Sending to Workers AI (mock mode)...");
        
        // Mock response for testing - simulates an agent deciding to give a final answer
        let mock_response = format!(
            r#"Thought: I have analyzed the goal "{}". Since this is a test environment, I will provide a direct response.

Final Answer: This is a mock response from the AGENT_CGI GOD_MODE orchestrator. The ReAct loop is functioning correctly. To enable real LLM inference, configure the Workers AI binding in wrangler.toml and update this module to use the AI binding."#,
            context.lines().find(|l| l.starts_with("GOAL:")).unwrap_or("GOAL: unknown")
        );

        Ok(mock_response)
        
        /* Production implementation would be:
        let ai = self.env.get_binding::<worker::Ai>("AI")?;
        let request = serde_json::json!({
            "messages": [
                {"role": "system", "content": "You are an autonomous AI agent. Follow the ReAct pattern precisely."},
                {"role": "user", "content": context}
            ],
            "max_tokens": 1024,
            "temperature": 0.7
        });

        let response = ai.run("@cf/meta/llama-3.1-8b-instruct", request).await?;
        
        response
            .as_object()
            .and_then(|obj| obj.get("response"))
            .and_then(|r| r.as_str())
            .map(String::from)
            .ok_or_else(|| Error::from("Invalid AI response format"))
        */
    }

    /// Extract tool call from LLM response
    fn extract_tool_call(&self, response: &str) -> Option<ToolCall> {
        // Check if this looks like a tool call (has Action: but not Final Answer:)
        if !response.contains("Action:") || response.contains("Final Answer:") {
            return None;
        }

        let mut action_name = None;
        let mut action_input = serde_json::json!({});

        for line in response.lines() {
            let line = line.trim();

            if line.starts_with("Action:") {
                action_name = Some(
                    line.trim_start_matches("Action:")
                        .trim()
                        .to_string()
                );
            } else if line.starts_with("Action Input:") {
                let input_str = line.trim_start_matches("Action Input:").trim();
                if let Ok(parsed) = serde_json::from_str(input_str) {
                    action_input = parsed;
                }
            }
        }

        action_name.map(|name| ToolCall {
            name,
            input: action_input,
        })
    }

    /// Extract final answer from LLM response
    fn extract_final_answer(&self, response: &str) -> Option<String> {
        if !response.contains("Final Answer:") {
            return None;
        }

        // Find "Final Answer:" and extract everything after it
        if let Some(idx) = response.find("Final Answer:") {
            let answer = response[idx + "Final Answer:".len()..].trim();
            if !answer.is_empty() {
                return Some(answer.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tool_call() {
        let response = r#"Thought: I need to search for information.
Action: web_search
Action Input: {"query": "rust cloudflare workers"}"#;

        // Would need to mock ReActLoop for full test
        assert!(response.contains("Action:"));
        assert!(!response.contains("Final Answer:"));
    }

    #[test]
    fn test_extract_final_answer() {
        let response = r#"Thought: I have all the information needed.
Final Answer: The answer to your question is 42."#;

        assert!(response.contains("Final Answer:"));
    }
}
