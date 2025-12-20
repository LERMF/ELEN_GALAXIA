use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentContext {
    pub memory: HashMap<String, String>,
    pub step_counter: u32,
    pub max_steps: u32,
}

impl AgentContext {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
            step_counter: 0,
            max_steps: 5, // ∞RSIP limit
        }
    }

    pub fn add_memory(&mut self, key: &str, value: &str) {
        self.memory.insert(key.to_string(), value.to_string());
    }
}

pub struct AgentGraph;

impl AgentGraph {
    /// Executes a single step of the ReAct loop
    pub fn step(ctx: &mut AgentContext, input: &str) -> String {
        ctx.step_counter += 1;
        
        if ctx.step_counter > ctx.max_steps {
            return "⛔ Max steps reached. Forcing convergence.".to_string();
        }

        // Simulating Agent thought process
        let thought = format!("Step {}: Analyzing '{}'", ctx.step_counter, input);
        ctx.add_memory(&format!("thought_{}", ctx.step_counter), &thought);

        format!("🔄 ReAct Cycle {}: Processed input via GraphFlow.", ctx.step_counter)
    }
}
