//! Flow composer for building executable transaction flows
//!
//! This module handles the composition of transaction flows using
//! semantic discovery and graph-based method chaining.

use super::{
    ApprovalError, FlowStep, MethodApprovalRequest, MethodApprover, MethodCandidate,
    TransactionFlow,
};
use crate::decimal::parse_amount_from_llm;
use crate::embedding::Embedding;
use crate::hnsw::HnswMemoryIndex;
use crate::llm::LlmClient;
use crate::models::Method;
use crate::queries::Queries;
use std::sync::Arc;
use surrealdb::RecordId;
use tracing::{debug, error, info, trace, warn};

/// Errors that can occur during flow composition
#[derive(Debug, thiserror::Error)]
pub enum CompositionError {
    #[error("Method selection failed for step {step} with pattern '{pattern}': {reason}")]
    MethodSelectionFailed {
        step: usize,
        pattern: String,
        reason: String,
    },

    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(#[from] crate::embedding::EmbeddingError),

    #[error("Database query failed: {0}")]
    DatabaseFailed(#[from] anyhow::Error),

    #[error("LLM approval failed: {0}")]
    ApprovalFailed(#[from] ApprovalError),
}

/// Flow composer for building executable transaction pipelines
pub struct FlowComposer {
    queries: Arc<Queries>,
    embedding: Arc<Embedding>,
    method_approver: MethodApprover,
    similarity_threshold: f32,
    ambiguity_threshold: f32,
}

impl FlowComposer {
    pub fn new(queries: Arc<Queries>, embedding: Arc<Embedding>, llm_client: LlmClient) -> Self {
        info!("🎼 Creating flow composer with LLM approval");
        Self {
            queries,
            embedding,
            method_approver: MethodApprover::new(llm_client),
            similarity_threshold: 0.7,
            ambiguity_threshold: 0.1,
        }
    }

    /// Create composer with custom thresholds
    pub fn with_thresholds(
        queries: Arc<Queries>,
        embedding: Arc<Embedding>,
        llm_client: LlmClient,
        similarity_threshold: f32,
        ambiguity_threshold: f32,
    ) -> Self {
        info!("🎼 Creating flow composer with custom thresholds");
        Self {
            queries,
            embedding,
            method_approver: MethodApprover::new(llm_client),
            similarity_threshold,
            ambiguity_threshold,
        }
    }

    /// Compose an executable flow from a transaction flow
    pub async fn compose_flow(
        &self,
        transaction_flow: &TransactionFlow,
        hnsw_index: &mut HnswMemoryIndex<'_>,
    ) -> Result<ComposedFlow, anyhow::Error> {
        info!("🎼 Composing flow from transaction pipeline");
        debug!("📋 Pipeline has {} steps", transaction_flow.pipeline.len());

        let mut composed_steps = Vec::new();

        for (index, step) in transaction_flow.pipeline.iter().enumerate() {
            debug!("🔍 Composing step {}: {}", index + 1, step.action);

            // Find method using semantic search
            let method = self.find_method_for_step(step, hnsw_index).await?;

            // Validate port compatibility with previous step
            if index > 0 {
                self.validate_port_compatibility(&composed_steps[index - 1], &method)
                    .await?;
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

    /// Compose an executable flow from a transaction flow using pre-computed search results
    /// This method is designed to work around the HNSW index not being Send
    pub async fn compose_flow_with_search_results(
        &self,
        transaction_flow: &TransactionFlow,
        search_results: &[Vec<(surrealdb::RecordId, f32)>],
    ) -> Result<ComposedFlow, anyhow::Error> {
        info!("🎼 Composing flow from transaction pipeline with pre-computed search results");
        debug!("📋 Pipeline has {} steps", transaction_flow.pipeline.len());

        if transaction_flow.pipeline.len() != search_results.len() {
            return Err(anyhow::anyhow!(
                "Mismatch between pipeline steps ({}) and search results ({})",
                transaction_flow.pipeline.len(),
                search_results.len()
            ));
        }

        let mut composed_steps = Vec::new();

        for (index, (step, step_search_results)) in transaction_flow
            .pipeline
            .iter()
            .zip(search_results.iter())
            .enumerate()
        {
            debug!("🔍 Composing step {}: {}", index + 1, step.action);

            // Find method using pre-computed search results
            let method = self
                .find_method_for_step_with_results(step, step_search_results)
                .await?;

            // Validate port compatibility with previous step
            if index > 0 {
                self.validate_port_compatibility(&composed_steps[index - 1], &method)
                    .await?;
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

    /// Find method for a flow step using pre-computed search results with LLM approval for ambiguity
    async fn find_method_for_step_with_results(
        &self,
        step: &FlowStep,
        search_results: &[(surrealdb::RecordId, f32)],
    ) -> Result<Method, CompositionError> {
        debug!(
            "🔍 Searching for method: {} using {} pre-computed results",
            step.method_pattern,
            search_results.len()
        );

        // Convert to MethodCandidate objects
        let mut candidates = Vec::new();
        for (method_id, similarity) in search_results {
            if *similarity < self.similarity_threshold {
                continue; // Skip low-quality matches
            }

            if let Ok(method) = self.get_method_by_id(method_id).await {
                candidates.push(MethodCandidate {
                    method,
                    similarity_score: *similarity,
                    reasoning: format!("Semantic similarity: {:.3}", similarity),
                });
            }
        }

        // Check for ambiguity and request LLM approval if needed
        if self.is_ambiguous(&candidates) {
            info!("🤖 Ambiguous method match detected, requesting LLM approval");
            let approved_method = self.request_llm_approval(step, &candidates).await?;
            return Ok(approved_method);
        }

        // Try exact match first (existing logic)
        for candidate in &candidates {
            if candidate.method.program.export == step.method_pattern {
                debug!(
                    "✅ Found exact method match: {}",
                    candidate.method.program.export
                );
                return Ok(candidate.method.clone());
            }
        }

        // Try pattern matching (existing logic)
        for candidate in &candidates {
            if self.method_matches_pattern(&candidate.method, &step.method_pattern) {
                debug!(
                    "✅ Found pattern match: {} (similarity: {:.3})",
                    candidate.method.program.export, candidate.similarity_score
                );
                return Ok(candidate.method.clone());
            }
        }

        error!("❌ No method found for pattern: {}", step.method_pattern);
        Err(CompositionError::MethodSelectionFailed {
            step: step.step,
            pattern: step.method_pattern.clone(),
            reason: format!(
                "No matching methods found among {} candidates",
                candidates.len()
            ),
        })
    }

    /// Find method for a flow step using semantic search with LLM approval for ambiguity
    async fn find_method_for_step(
        &self,
        step: &FlowStep,
        hnsw_index: &mut HnswMemoryIndex<'_>,
    ) -> Result<Method, CompositionError> {
        debug!("🔍 Searching for method: {}", step.method_pattern);

        // Generate embedding for semantic hook
        let embedding = self.embedding.embed(&step.semantic_hook).await?;
        trace!("🧠 Generated embedding for semantic hook");

        // Search HNSW index for similar methods (get more candidates)
        let search_results = hnsw_index.search(&embedding, 10);
        debug!("🔍 Found {} candidate methods", search_results.len());

        // Convert to MethodCandidate objects
        let mut candidates = Vec::new();
        for (method_id, similarity) in search_results {
            if similarity < self.similarity_threshold {
                continue; // Skip low-quality matches
            }

            if let Ok(method) = self.get_method_by_id(&method_id).await {
                candidates.push(MethodCandidate {
                    method,
                    similarity_score: similarity,
                    reasoning: format!("Semantic similarity: {:.3}", similarity),
                });
            }
        }

        // Check for ambiguity and request LLM approval if needed
        if self.is_ambiguous(&candidates) {
            info!("🤖 Ambiguous method match detected, requesting LLM approval");
            let approved_method = self.request_llm_approval(step, &candidates).await?;
            return Ok(approved_method);
        }

        // Try exact match first (existing logic)
        for candidate in &candidates {
            if candidate.method.program.export == step.method_pattern {
                debug!(
                    "✅ Found exact method match: {}",
                    candidate.method.program.export
                );
                return Ok(candidate.method.clone());
            }
        }

        // Try pattern matching (existing logic)
        for candidate in &candidates {
            if self.method_matches_pattern(&candidate.method, &step.method_pattern) {
                debug!(
                    "✅ Found pattern match: {} (similarity: {:.3})",
                    candidate.method.program.export, candidate.similarity_score
                );
                return Ok(candidate.method.clone());
            }
        }

        error!("❌ No method found for pattern: {}", step.method_pattern);
        Err(CompositionError::MethodSelectionFailed {
            step: step.step,
            pattern: step.method_pattern.clone(),
            reason: format!(
                "No matching methods found among {} candidates",
                candidates.len()
            ),
        })
    }

    /// Check if method candidates are ambiguous (similar scores)
    fn is_ambiguous(&self, candidates: &[MethodCandidate]) -> bool {
        if candidates.len() < 2 {
            return false;
        }

        // Sort by similarity score
        let mut sorted = candidates.to_vec();
        sorted.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        let top_score = sorted[0].similarity_score;
        let second_score = sorted[1].similarity_score;

        // Check if scores are close enough to be ambiguous
        let score_diff = top_score - second_score;
        let is_ambiguous =
            score_diff < self.ambiguity_threshold && top_score > self.similarity_threshold;

        if is_ambiguous {
            debug!(
                "🔍 Ambiguity detected: top={:.3}, second={:.3}, diff={:.3}",
                top_score, second_score, score_diff
            );
        }

        is_ambiguous
    }

    /// Request LLM approval for ambiguous method selection
    async fn request_llm_approval(
        &self,
        step: &FlowStep,
        candidates: &[MethodCandidate],
    ) -> Result<Method, CompositionError> {
        let approval_request = MethodApprovalRequest {
            original_query: step.semantic_hook.clone(),
            step_intent: format!(
                "{} {} to {}",
                step.action,
                step.from_asset.as_deref().unwrap_or("unknown"),
                step.to_asset.as_deref().unwrap_or("unknown")
            ),
            candidates: candidates.to_vec(),
            flow_context: None, // TODO: Add multi-step context
            expected_pattern: Some(step.method_pattern.clone()),
        };

        let approval = self
            .method_approver
            .request_approval_with_fallback(&approval_request)
            .await?;

        // Find the approved method
        for candidate in candidates {
            if candidate.method.program.export == approval.selected_method_export {
                info!(
                    "✅ LLM approved method: {} - {}",
                    approval.selected_method_export, approval.reasoning
                );
                return Ok(candidate.method.clone());
            }
        }

        // If approved method not found in candidates, fall back to highest similarity
        warn!("⚠️  LLM approved method not found in candidates, using highest similarity");
        Ok(candidates[0].method.clone())
    }

    /// Get method by ID from database
    async fn get_method_by_id(&self, method_id: &RecordId) -> Result<Method, anyhow::Error> {
        trace!("🔍 Fetching method from database: {}", method_id);

        let method = self.queries.get_method(method_id.clone()).await?;

        debug!("✅ Retrieved method: {}", method.program.export);
        Ok(method)
    }

    /// Check if method matches the expected pattern
    fn method_matches_pattern(&self, method: &Method, pattern: &str) -> bool {
        // Simple pattern matching for now
        // TODO: Implement more sophisticated pattern matching
        method.program.export.contains(&pattern.to_lowercase())
            || pattern.contains(&method.program.export.to_lowercase())
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
            warn!(
                "⚠️  Port type mismatch: {:?} -> {:?}",
                previous_output_type, current_input_type
            );
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
                output.as_str() == input.as_str()
                    || output.as_str() == "type://u128" && input.as_str() == "type://u128"
                    || output.as_str().contains("asset") && input.as_str().contains("asset")
            }
            _ => true, // If types are not specified, assume compatibility
        }
    }

    /// Prepare input data for a step
    fn prepare_input_data(
        &self,
        step: &FlowStep,
        previous_steps: &[ComposedStep],
    ) -> Result<serde_json::Value, anyhow::Error> {
        match step.amount.as_str() {
            "previous_output" => {
                if let Some(_prev_step) = previous_steps.last() {
                    // TODO: Get actual output from previous step
                    // For now, return placeholder
                    Ok(serde_json::json!({"amount": "previous_output"}))
                } else {
                    Err(anyhow::anyhow!(
                        "No previous step for 'previous_output' amount"
                    ))
                }
            }
            "user_specified" => Ok(serde_json::json!({"amount": "user_specified"})),
            amount_str => {
                // Try to parse as decimal amount from LLM
                match parse_amount_from_llm(amount_str) {
                    Ok(amount_u128) => {
                        debug!(
                            "💰 Parsed amount from LLM: {} -> {}",
                            amount_str, amount_u128
                        );
                        Ok(serde_json::json!({"amount": amount_u128}))
                    }
                    Err(e) => {
                        debug!(
                            "⚠️  Could not parse amount '{}' as decimal: {}, using as string",
                            amount_str, e
                        );
                        Ok(serde_json::json!({"amount": amount_str}))
                    }
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
