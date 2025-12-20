//! # State Management Module
//!
//! Manages agent session state with D1 persistence and Vectorize RAG context.
//!
//! ## State Components
//! - Session ID: Unique identifier for the agent invocation
//! - Goal: The current task objective
//! - Memory: Key-value context from previous steps
//! - Step count: Iteration tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::*;

/// Agent state containing session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Unique session identifier
    pub session_id: String,
    /// Current goal/objective
    pub goal: String,
    /// Key-value memory store
    pub memory: HashMap<String, String>,
    /// Current step in the ReAct loop
    pub step_count: u32,
    /// Timestamp of session creation
    pub created_at: u64,
}

impl AgentState {
    /// Create a new agent state
    pub fn new(goal: &str) -> Self {
        let session_id = generate_session_id();
        let now = (js_sys::Date::now() / 1000.0) as u64;

        console_log!("📦 Creating AgentState: session_id={}", session_id);

        Self {
            session_id,
            goal: goal.to_string(),
            memory: HashMap::new(),
            step_count: 0,
            created_at: now,
        }
    }

    /// Add a key-value pair to memory
    pub fn remember(&mut self, key: &str, value: &str) {
        self.memory.insert(key.to_string(), value.to_string());
    }

    /// Recall a value from memory
    #[allow(dead_code)]
    pub fn recall(&self, key: &str) -> Option<&String> {
        self.memory.get(key)
    }

    /// Increment step counter
    #[allow(dead_code)]
    pub fn step(&mut self) {
        self.step_count += 1;
    }
}

/// Generate a unique session ID
fn generate_session_id() -> String {
    let now = js_sys::Date::now() as u64;
    let random: u32 = (js_sys::Math::random() * u32::MAX as f64) as u32;
    format!("agent_{:016x}_{:08x}", now, random)
}

/// D1 SQL schema for agent_logs table
/// Run this once to create the table:
/// ```sql
/// CREATE TABLE IF NOT EXISTS agent_logs (
///     id INTEGER PRIMARY KEY AUTOINCREMENT,
///     session_id TEXT NOT NULL UNIQUE,
///     goal TEXT NOT NULL,
///     memory TEXT DEFAULT '{}',
///     step_count INTEGER DEFAULT 0,
///     status TEXT DEFAULT 'started',
///     result TEXT,
///     created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
///     completed_at DATETIME
/// );
///
/// CREATE INDEX idx_agent_logs_session ON agent_logs(session_id);
/// CREATE INDEX idx_agent_logs_status ON agent_logs(status);
/// ```
#[allow(dead_code)]
pub const D1_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS agent_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,
    goal TEXT NOT NULL,
    memory TEXT DEFAULT '{}',
    step_count INTEGER DEFAULT 0,
    status TEXT DEFAULT 'started',
    result TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_agent_logs_session ON agent_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_logs_status ON agent_logs(status);
"#;
