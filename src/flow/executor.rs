//! Flow executor for running composed transaction flows
//!
//! This module handles execution of composed flows with proper
//! state management and error handling.

use tracing::{info, debug, trace, warn, error};
use crate::ledger::Ledger;
use crate::executor::exec_local;
use crate::decimal::format_amount_for_display;
use super::{ExecutionResult, StepResult};
use crate::flow::composer::{ComposedFlow, ComposedStep};

/// Flow executor for running composed transaction flows
pub struct FlowExecutor {
    ledger: Ledger,
}

impl FlowExecutor {
    pub fn new(ledger: Ledger) -> Self {
        info!("⚡ Creating flow executor");
        Self { ledger }
    }

    /// Execute a composed flow and return results
    pub async fn execute_flow(&mut self, flow: &ComposedFlow) -> ExecutionResult {
        info!("⚡ Executing flow with {} steps", flow.steps.len());
        debug!("🎯 Flow intent: {}", flow.intent);

        let mut steps = Vec::new();
        let mut current_state: Option<serde_json::Value> = None;

        for (index, composed_step) in flow.steps.iter().enumerate() {
            debug!("⚡ Executing step {}: {}", index + 1, composed_step.method.program.export);
            
            let step_result = self.execute_step(composed_step, current_state.as_ref()).await;
            
            match step_result {
                Ok(result) => {
                    debug!("✅ Step {} completed successfully", index + 1);
                    trace!("📤 Step output: {}", result.output);
                    current_state = Some(result.output.clone());
                    steps.push(result);
                }
                Err(e) => {
                    error!("❌ Step {} failed: {}", index + 1, e);
                    return ExecutionResult {
                        success: false,
                        steps,
                        final_state: current_state,
                        error: Some(format!("Step {} failed: {}", index + 1, e)),
                    };
                }
            }
        }

        info!("✅ Flow execution completed successfully");
        ExecutionResult {
            success: true,
            steps,
            final_state: current_state,
            error: None,
        }
    }

    /// Execute a single step in the flow
    async fn execute_step(
        &mut self,
        composed_step: &ComposedStep,
        previous_state: Option<&serde_json::Value>,
    ) -> Result<StepResult, anyhow::Error> {
        trace!("⚡ Executing method: {}", composed_step.method.program.export);

        // Prepare input data
        let input_data = self.prepare_step_input(composed_step, previous_state)?;
        trace!("📥 Step input: {}", input_data);

        // Execute the method using the existing executor
        let output = exec_local(&composed_step.method, &mut self.ledger, input_data.clone());
        trace!("📤 Step output: {}", output);

        // Create step result
        let step_result = StepResult {
            step: composed_step.step,
            method_name: composed_step.method.program.export.clone(),
            input: input_data,
            output: output.clone(),
            success: true,
            error: None,
        };

        debug!("✅ Step {} executed successfully", composed_step.step);
        Ok(step_result)
    }

    /// Prepare input data for a step, handling previous outputs
    fn prepare_step_input(
        &self,
        composed_step: &ComposedStep,
        previous_state: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, anyhow::Error> {
        trace!("🔧 Preparing input for step {}", composed_step.step);

        // Start with the composed step's input data
        let mut input_data = composed_step.input_data.clone();

        // If we have previous state and this step expects "previous_output"
        if let Some(prev_state) = previous_state {
            // Check if this step needs previous output
            if self.step_needs_previous_output(&composed_step) {
                // Extract relevant data from previous output
                if let Some(extracted) = self.extract_amount_from_output(prev_state) {
                    debug!("💰 Using previous output amount: {}", extracted);
                    if let Ok(amount) = extracted.parse::<u128>() {
                        input_data["amount"] = serde_json::Value::Number(
                            serde_json::Number::from(amount as i64)
                        );
                        debug!("💰 Converted previous output: {} -> {} (internal)", extracted, amount);
                    }
                } else {
                    warn!("⚠️  Could not extract amount from previous output");
                }
            }
        }

        trace!("📋 Final input data: {}", input_data);
        Ok(input_data)
    }

    /// Check if a step needs previous output as input
    fn step_needs_previous_output(&self, composed_step: &ComposedStep) -> bool {
        // Check if the input data contains "previous_output" placeholder
        composed_step.input_data.get("amount")
            .and_then(|v| v.as_str())
            .map(|s| s == "previous_output")
            .unwrap_or(false)
    }

    /// Extract amount from previous step output
    fn extract_amount_from_output(&self, output: &serde_json::Value) -> Option<String> {
        trace!("🔍 Extracting amount from output: {}", output);

        // Try different possible output formats
        if let Some(obj) = output.as_object() {
            // Check for "received_amount" field (from swap operations)
            if let Some(amount) = obj.get("received_amount") {
                debug!("💰 Found received_amount: {}", amount);
                return Some(amount.to_string());
            }

            // Check for "amount" field
            if let Some(amount) = obj.get("amount") {
                debug!("💰 Found amount: {}", amount);
                return Some(amount.to_string());
            }
        }

        // If output is a string (like from ping), return it
        if let Some(s) = output.as_str() {
            debug!("📝 Found string output: {}", s);
            return Some(s.to_string());
        }

        // If output is a number, return it (this is the key fix for swap outputs)
        if let Some(n) = output.as_u64() {
            debug!("💰 Found number output: {} (internal u128 representation)", n);
            // Convert to decimal format for display/logging
            let amount_u128 = n as u128;
            let display_amount = format_amount_for_display(amount_u128);
            debug!("💰 Formatted for display: {} -> {}", amount_u128, display_amount);
            return Some(amount_u128.to_string());
        }

        // Also check for i64 numbers (JSON numbers can be i64)
        if let Some(n) = output.as_i64() {
            debug!("💰 Found i64 number output: {} (internal u128 representation)", n);
            // Convert to decimal format for display/logging
            let amount_u128 = n as u128;
            let display_amount = format_amount_for_display(amount_u128);
            debug!("💰 Formatted for display: {} -> {}", amount_u128, display_amount);
            return Some(amount_u128.to_string());
        }

        warn!("⚠️  Could not extract amount from output");
        None
    }

    /// Get current ledger state for debugging
    pub fn get_ledger_snapshot(&self) -> Vec<(String, u128)> {
        self.ledger.balances.clone()
    }

    /// Get current ledger state as Ledger
    pub fn get_ledger(&self) -> crate::ledger::Ledger {
        crate::ledger::Ledger {
            balances: self.ledger.balances.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_amount_from_output() {
        let executor = FlowExecutor::new(Ledger::new());

        // Test swap output with received_amount field
        let swap_output = serde_json::json!({"received_amount": 5000});
        let amount = executor.extract_amount_from_output(&swap_output);
        assert_eq!(amount, Some("5000".to_string()));

        // Test swap output with amount field
        let amount_output = serde_json::json!({"amount": 5000});
        let amount = executor.extract_amount_from_output(&amount_output);
        assert_eq!(amount, Some("5000".to_string()));

        // Test bare number output (this is the key fix for swap functions)
        let number_output = serde_json::json!(5000);
        let amount = executor.extract_amount_from_output(&number_output);
        assert_eq!(amount, Some("5000".to_string()));

        // Test i64 number output
        let i64_output = serde_json::json!(-5000);
        let amount = executor.extract_amount_from_output(&i64_output);
        assert_eq!(amount, Some("-5000".to_string()));

        // Test ping output
        let ping_output = serde_json::json!("pong: test");
        let amount = executor.extract_amount_from_output(&ping_output);
        assert_eq!(amount, Some("pong: test".to_string()));
    }
}