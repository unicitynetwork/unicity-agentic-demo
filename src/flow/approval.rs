//! LLM-based method approval for ambiguous semantic matches
//!
//! This module provides intelligent method selection when semantic search
//! returns multiple candidates with similar similarity scores, particularly
//! for directional operations like swaps (USDT→NEAR vs NEAR→USDT).

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use crate::llm::LlmClient;
use crate::models::Method;

/// A candidate method with similarity score and reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCandidate {
    pub method: Method,
    pub similarity_score: f32,
    pub reasoning: String,
}

/// Request for LLM method approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodApprovalRequest {
    pub original_query: String,
    pub step_intent: String,
    pub candidates: Vec<MethodCandidate>,
    pub flow_context: Option<String>,
    pub expected_pattern: Option<String>,
}

/// LLM response for method selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodApprovalResponse {
    pub selected_method_id: String,
    pub selected_method_export: String,
    pub reasoning: String,
    pub confidence: f32,
    pub alternative_suggestions: Vec<String>,
}

/// Errors that can occur during method approval
#[derive(Debug, thiserror::Error)]
pub enum ApprovalError {
    #[error("LLM approval request failed: {0}")]
    LlmRequestFailed(#[from] anyhow::Error),
    
    #[error("Failed to parse LLM response: {0}")]
    ResponseParseFailed(#[from] serde_json::Error),
    
    #[error("No fallback methods available")]
    NoFallbackAvailable,
}

/// Handles LLM-based method approval for ambiguous semantic matches
pub struct MethodApprover {
    llm_client: LlmClient,
}

impl MethodApprover {
    /// Create a new method approver with the given LLM client
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }

    /// Request LLM approval for method selection with fallback mechanisms
    pub async fn request_approval_with_fallback(&self, request: &MethodApprovalRequest) -> Result<MethodApprovalResponse, ApprovalError> {
        match self.request_approval(request).await {
            Ok(approval) => Ok(approval),
            Err(e) => {
                warn!("⚠️  LLM approval failed, attempting fallback: {}", e);
                
                // Fallback strategy 1: Use expected pattern if available
                if let Some(expected_pattern) = &request.expected_pattern {
                    if let Some(candidate) = request.candidates.iter()
                        .find(|c| c.method.program.export == *expected_pattern) {
                        info!("✅ Using expected pattern fallback: {}", expected_pattern);
                        return Ok(MethodApprovalResponse {
                            selected_method_id: candidate.method.id.to_string(),
                            selected_method_export: expected_pattern.clone(),
                            reasoning: format!("Fallback to expected pattern due to LLM failure: {}", e),
                            confidence: 0.6,
                            alternative_suggestions: vec![],
                        });
                    }
                }
                
                // Fallback strategy 2: Use highest similarity
                if let Some(best_candidate) = request.candidates.first() {
                    warn!("⚠️  Using highest similarity fallback: {} (confidence: 0.5)", 
                        best_candidate.method.program.export);
                    return Ok(MethodApprovalResponse {
                        selected_method_id: best_candidate.method.id.to_string(),
                        selected_method_export: best_candidate.method.program.export.clone(),
                        reasoning: format!("Fallback to highest similarity due to LLM failure: {}", e),
                        confidence: 0.5,
                        alternative_suggestions: vec![],
                    });
                }
                
                // Fallback strategy 3: No candidates available
                Err(ApprovalError::NoFallbackAvailable)
            }
        }
    }

    /// Request LLM approval for method selection
    pub async fn request_approval(&self, request: &MethodApprovalRequest) -> Result<MethodApprovalResponse, anyhow::Error> {
        info!("🤖 Requesting LLM approval for method selection");
        debug!("📋 Step intent: {}", request.step_intent);
        debug!("🔍 Candidates: {}", request.candidates.len());

        let prompt = self.create_approval_prompt(request);
        let response = self.llm_client.generate_response(&prompt).await;

        match response.success {
            true => {
                debug!("✅ Received LLM approval response");
                let approval: MethodApprovalResponse = serde_json::from_str(&response.response)
                    .map_err(|e| anyhow::anyhow!("Failed to parse approval response: {}", e))?;
                
                info!("✅ LLM selected method: {} (confidence: {:.2})", 
                    approval.selected_method_export, approval.confidence);
                Ok(approval)
            }
            false => {
                error!("❌ LLM approval failed: {:?}", response.error);
                Err(anyhow::anyhow!("LLM approval failed: {:?}", response.error))
            }
        }
    }

    /// Create the LLM prompt for method approval
    fn create_approval_prompt(&self, request: &MethodApprovalRequest) -> String {
        format!(r#"
You are a method selection specialist for a financial transaction system. 

ORIGINAL USER QUERY: "{}"
CURRENT STEP INTENT: "{}"
EXPECTED METHOD PATTERN: {}

CANDIDATE METHODS:
{}

CONTEXT: {}

TASK: Select the MOST appropriate method for this specific step. Consider:
1. Direction of the transaction (from/to assets) - THIS IS CRITICAL
2. User's explicit intent and wording
3. Semantic meaning of the action
4. Method descriptions and labels
5. Method export names and patterns

For swap operations, pay special attention to:
- "swap X to Y" means method should be "swap_x_to_y"
- "convert X to Y" means method should be "swap_x_to_y" 
- Direction matters: USDT to NEAR ≠ NEAR to USDT

Respond with JSON only (no code blocks):
{{
  "selected_method_id": "method_id_here",
  "selected_method_export": "exact_method_export_name", 
  "reasoning": "Clear explanation of why this method matches the user's intent, especially direction",
  "confidence": 0.95,
  "alternative_suggestions": ["other_method_if_relevant"]
}}
"#,
            request.original_query,
            request.step_intent,
            request.expected_pattern.as_deref().unwrap_or("none"),
            self.format_candidates(&request.candidates),
            request.flow_context.as_deref().unwrap_or("No additional context")
        )
    }

    /// Format candidate methods for the LLM prompt
    fn format_candidates(&self, candidates: &[MethodCandidate]) -> String {
        candidates
            .iter()
            .enumerate()
            .map(|(i, candidate)| {
                format!(
                    "{}. Export: {}\n   Label: {}\n   Description: {}\n   Similarity: {:.3}\n   ID: {}",
                    i + 1,
                    candidate.method.program.export,
                    candidate.method.label,
                    candidate.method.description,
                    candidate.similarity_score,
                    candidate.method.id
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateMethod, CreatePort, ProgramRef, ProgramAbi, ExecKind, Visibility};
    use fluent_uri::Uri;

    fn create_mock_swap_method(from: &str, to: &str) -> Method {
        Method {
            id: surrealdb::RecordId::from(("method", format!("swap_{}_to_{}", from, to))),
            agent: surrealdb::RecordId::from(("agent", "swap")),
            label: format!("Swap {}→{}", from, to),
            version: "1.0.0".to_string(),
            visibility: Visibility::Public,
            exec_kind: ExecKind::Local,
            description: format!("Swap assets from {} to {}. Expects {{\"amount\"}}.", from, to),
            in_port: CreatePort {
                label: format!("in.swap-{}-{}", from, to),
                description: format!("Accepts ConversionRequest from {} to {}.", from, to),
                channel: crate::models::Channel::Call,
                type_uri: Some("type://u128".parse().unwrap()),
                end_point: format!("agent://SwapAgent/swap#{}_{}_in", from, to).parse().unwrap(),
            },
            out_port: CreatePort {
                label: format!("out.swap-{}-{}", from, to),
                description: format!("Emits AssetAmount of {}.", to),
                channel: crate::models::Channel::Call,
                type_uri: Some("type://u128".parse().unwrap()),
                end_point: format!("agent://SwapAgent/swap#{}_{}_out", from, to).parse().unwrap(),
            },
            program: ProgramRef {
                module_uri: format!("local://SwapAgent/swap_{}_{}", from, to).parse().unwrap(),
                export: format!("swap_{}_to_{}", from, to),
                abi: ProgramAbi::LocalFn,
                checksum: "demo".to_string(),
            },
            embedding: vec![0.0; 1024], // Mock embedding
        }
    }

    #[test]
    fn test_format_candidates() {
        let approver = MethodApprover {
            llm_client: LlmClient::new("test_key".to_string()),
        };

        let candidates = vec![
            MethodCandidate {
                method: create_mock_swap_method("usdt", "near"),
                similarity_score: 0.87,
                reasoning: "USDT to NEAR conversion".to_string(),
            },
            MethodCandidate {
                method: create_mock_swap_method("near", "usdt"),
                similarity_score: 0.85,
                reasoning: "NEAR to USDT conversion".to_string(),
            },
        ];

        let formatted = approver.format_candidates(&candidates);
        
        assert!(formatted.contains("swap_usdt_to_near"));
        assert!(formatted.contains("swap_near_to_usdt"));
        assert!(formatted.contains("0.87"));
        assert!(formatted.contains("0.85"));
    }

    #[test]
    fn test_create_approval_prompt() {
        let approver = MethodApprover {
            llm_client: LlmClient::new("test_key".to_string()),
        };

        let request = MethodApprovalRequest {
            original_query: "swap 100 USDT to NEAR".to_string(),
            step_intent: "swap USDT to NEAR".to_string(),
            candidates: vec![
                MethodCandidate {
                    method: create_mock_swap_method("usdt", "near"),
                    similarity_score: 0.87,
                    reasoning: "USDT to NEAR conversion".to_string(),
                },
            ],
            flow_context: None,
            expected_pattern: Some("swap_usdt_to_near".to_string()),
        };

        let prompt = approver.create_approval_prompt(&request);
        
        assert!(prompt.contains("swap 100 USDT to NEAR"));
        assert!(prompt.contains("swap USDT to NEAR"));
        assert!(prompt.contains("swap_usdt_to_near"));
        assert!(prompt.contains("Direction of the transaction"));
    }
}