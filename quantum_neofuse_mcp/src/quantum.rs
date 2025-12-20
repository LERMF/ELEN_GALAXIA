use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuantumBranch {
    pub id: String,
    pub probability: f64,
    pub state: String, // "Superposition", "Collapsed"
    pub value: Option<f64>,
}

impl QuantumBranch {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            probability: 0.5,
            state: "Superposition".to_string(),
            value: None,
        }
    }

    /// Simulates a measurement collapsing the wave function
    pub fn measure(&mut self) -> f64 {
        let mut rng = rand::thread_rng();
        let outcome: f64 = rng.gen();
        
        self.value = Some(outcome);
        self.state = "Collapsed".to_string();
        
        // "Entanglement" check mock
        if outcome > 0.95 {
            println!("🔒 Quantum Entanglement Verified for branch {}", self.id);
        }

        outcome
    }

    /// Simulates Ψ-Branching logic for decision trees
    pub fn psi_branch(branches: usize) -> Vec<QuantumBranch> {
        let mut result = Vec::new();
        for i in 0..branches {
            result.push(QuantumBranch::new(&format!("branch_{}", i)));
        }
        result
    }
}
