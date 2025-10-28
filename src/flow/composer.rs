//! Flow composer for building executable transaction flows
//!
//! This module handles the composition of transaction flows using
//! semantic discovery and graph-based method chaining.

use std::sync::Arc;
use surrealdb::RecordId;
use tracing::{info, debug, trace, warn, error};
use crate::hnsw::HnswMemoryIndex;
use crate::embedding::Embedding;
use crate::queries::Queries;
use crate::models::Method;
use super::{TransactionFlow, FlowStep, ExecutionResult, StepResult};

/// Flow composer for building executable transaction pipelines
pub struct FlowComposer {
    queries: Arc<Queries>,
    embedding: Arc<Embedding>,
    // hnsw_index: Arc<HnswMemoryIndex<'static>>,
}

impl FlowComposer {
    pub fn new(
        queries: Arc<Queries>,
        embedding: Arc<Embedding>,
        // hnsw_index: Arc<HnswMemoryIndex<'static>>,
    ) -> Self {
        info!("🎼 Creating flow composer");
        Self {
            queries,
            embedding,
            // hnsw_index,
        }
    }

    /// Compose an executable flow from a transaction flow
    pub async fn compose_flow(&self, transaction_flow: &TransactionFlow, hnsw_index: &mut HnswMemoryIndex<'_>) -> Result<ComposedFlow, anyhow::Error> {
        info!("🎼 Composing flow from transaction pipeline");
        debug!("📋 Pipeline has {} steps", transaction_flow.pipeline.len());

        let mut composed_steps = Vec::new();

        for (index, step) in transaction_flow.pipeline.iter().enumerate() {
            debug!("🔍 Composing step {}: {}", index + 1, step.action);
            
            // Find method using semantic search
            let method = self.find_method_for_step(step, hnsw_index).await?;
            
            // Validate port compatibility with previous step
            if index > 0 {
                self.validate_port_compatibility(&composed_steps[index - 1], &method).await?;
            }

            let composed_step = ComposedStep {
                step: step.step,
                method,
                input_data: self.prepare_input_data(step, &composed_steps)?,
                semantic_hook: step.semantic_hook.clone(),
            };

            composed_steps.push(composed_step);
            trace!("✅ Step {} composed successfully", index + 1);
        }

        info!("✅ Flow composed with {} steps", composed_steps.len());
        Ok(ComposedFlow {
            steps: composed_steps,
            intent: transaction_flow.intent.clone(),
            expected_outcome: transaction_flow.expected_outcome.clone(),
        })
    }

    /// Find method for a flow step using semantic search
    async fn find_method_for_step(&self, step: &FlowStep, hnsw_index: &mut HnswMemoryIndex<'_>) -> Result<Method, anyhow::Error> {
        debug!("🔍 Searching for method: {}", step.method_pattern);
        
        // Generate embedding for semantic hook
        let embedding = self.embedding.embed(&step.semantic_hook).await?;
        trace!("🧠 Generated embedding for semantic hook");

        // Search HNSW index for similar methods
        let search_results = hnsw_index.search(&embedding, 5);
        debug!("🔍 Found {} candidate methods", search_results.len());

        // Find exact match by method pattern first
        for (method_id, similarity) in &search_results {
            trace!("🔍 Checking method {} with similarity {}", method_id, similarity);
            
            if let Ok(method) = self.get_method_by_id(&method_id).await {
                if method.program.export == step.method_pattern {
                    debug!("✅ Found exact method match: {}", method.program.export);
                    return Ok(method);
                }
            }
        }

        // If no exact match, try to find by pattern matching
        for (method_id, similarity) in &search_results {
            if let Ok(method) = self.get_method_by_id(&method_id).await {
                if self.method_matches_pattern(&method, &step.method_pattern) {
                    debug!("✅ Found pattern match: {} (similarity: {})", method.program.export, similarity);
                    return Ok(method);
                }
            }
        }

        error!("❌ No method found for pattern: {}", step.method_pattern);
        Err(anyhow::anyhow!("No method found for pattern: {}", step.method_pattern))
    }

    /// Get method by ID from database
    async fn get_method_by_id(&self, method_id: &RecordId) -> Result<Method, anyhow::Error> {
        trace!("🔍 Fetching method from database: {}", method_id);
        
        // TODO: Implement actual database query
        // For now, create a mock method for testing
        let mock_method = Method {
            id: method_id.clone(),
            agent: method_id.clone(), // placeholder
            label: "Mock Method".to_string(),
            version: "1.0.0".to_string(),
            visibility: crate::models::Visibility::Public,
            exec_kind: crate::models::ExecKind::Local,
            description: "Mock method for testing".to_string(),
            in_port: crate::models::CreatePort {
                label: "input".to_string(),
                description: "Input port".to_string(),
                channel: crate::models::Channel::Call,
                type_uri: Some("type://u128".parse().unwrap()),
                end_point: "agent://mock/input".parse().unwrap(),
            },
            out_port: crate::models::CreatePort {
                label: "output".to_string(),
                description: "Output port".to_string(),
                channel: crate::models::Channel::Call,
                type_uri: Some("type://u128".parse().unwrap()),
                end_point: "agent://mock/output".parse().unwrap(),
            },
            program: crate::models::ProgramRef {
                module_uri: "local://mock/method".parse().unwrap(),
                export: "mock_method".to_string(),
                abi: crate::models::ProgramAbi::LocalFn,
                checksum: "mock".to_string(),
            },
            embedding: vec![0.0; 1024], // placeholder embedding
        };
        
        debug!("✅ Retrieved method: {}", mock_method.program.export);
        Ok(mock_method)
    }

    /// Check if method matches the expected pattern
    fn method_matches_pattern(&self, method: &Method, pattern: &str) -> bool {
        // Simple pattern matching for now
        // TODO: Implement more sophisticated pattern matching
        method.program.export.contains(&pattern.to_lowercase()) ||
        pattern.contains(&method.program.export.to_lowercase())
    }

    /// Validate port compatibility between consecutive steps
    async fn validate_port_compatibility(
        &self,
        previous_step: &ComposedStep,
        current_method: &Method,
    ) -> Result<(), anyhow::Error> {
        debug!("🔗 Validating port compatibility between steps");
        
        // Get output type of previous step
        let previous_output_type = &previous_step.method.out_port.type_uri;
        trace!("📤 Previous output type: {:?}", previous_output_type);
        
        // Get input type of current step
        let current_input_type = &current_method.in_port.type_uri;
        trace!("📥 Current input type: {:?}", current_input_type);
        
        // Check compatibility
        if self.are_types_compatible(previous_output_type, current_input_type) {
            debug!("✅ Port types are compatible");
            Ok(())
        } else {
            warn!("⚠️  Port type mismatch: {:?} -> {:?}", previous_output_type, current_input_type);
            Err(anyhow::anyhow!("Port type mismatch between steps"))
        }
    }

    /// Check if two types are compatible
    fn are_types_compatible(
        &self,
        output_type: &Option<fluent_uri::Uri<String>>,
        input_type: &Option<fluent_uri::Uri<String>>,
    ) -> bool {
        match (output_type, input_type) {
            (Some(output), Some(input)) => {
                // Simple URI matching for now
                // TODO: Implement more sophisticated type compatibility checking
                output.as_str() == input.as_str() ||
                output.as_str() == "type://u128" && input.as_str() == "type://u128" ||
                output.as_str().contains("asset") && input.as_str().contains("asset")
            }
            _ => true, // If types are not specified, assume compatibility
        }
    }

    /// Prepare input data for a step
    fn prepare_input_data(&self, step: &FlowStep, previous_steps: &[ComposedStep]) -> Result<serde_json::Value, anyhow::Error> {
        match step.amount.as_str() {
            "previous_output" => {
                if let Some(_prev_step) = previous_steps.last() {
                    // TODO: Get actual output from previous step
                    // For now, return placeholder
                    Ok(serde_json::json!({"amount": "previous_output"}))
                } else {
                    Err(anyhow::anyhow!("No previous step for 'previous_output' amount"))
                }
            }
            "user_specified" => {
                Ok(serde_json::json!({"amount": "user_specified"}))
            }
            amount_str => {
                // Try to parse as number
                if let Ok(amount) = amount_str.parse::<u128>() {
                    Ok(serde_json::json!({"amount": amount}))
                } else {
                    Ok(serde_json::json!({"amount": amount_str}))
                }
            }
        }
    }
}

/// A composed flow ready for execution
#[derive(Debug, Clone)]
pub struct ComposedFlow {
    pub steps: Vec<ComposedStep>,
    pub intent: String,
    pub expected_outcome: String,
}

/// A single composed step in the flow
#[derive(Debug, Clone)]
pub struct ComposedStep {
    pub step: usize,
    pub method: Method,
    pub input_data: serde_json::Value,
    pub semantic_hook: String,
}