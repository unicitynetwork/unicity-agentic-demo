//! Transaction flow parser for converting natural language to executable flows
//!
//! This module handles the LLM-based parsing of user queries into
//! structured transaction flows using semantic understanding.

use serde::{Deserialize, Serialize};
use tracing::{info, debug, trace, warn, error};
use crate::llm::LlmClient;
use crate::decimal::{format_amount_for_llm, parse_amount_from_llm, DECIMAL_PLACES};

/// Transaction flow parsed from natural language query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFlow {
    pub tool: String,
    pub intent: String,
    pub pipeline: Vec<FlowStep>,
    pub expected_outcome: String,
}

/// Individual step in the transaction pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub step: usize,
    pub action: String,
    pub from_asset: Option<String>,
    pub to_asset: Option<String>,
    pub amount: String, // "number" or "previous_output" or "user_specified"
    pub semantic_hook: String,
    pub method_pattern: String,
}

/// Parse natural language query into transaction flow using LLM
pub async fn parse_transaction_flow(
    query: &str,
    llm_client: &LlmClient,
) -> Result<TransactionFlow, anyhow::Error> {
    info!("🔍 Parsing transaction flow from query: {}", query);
    
    let prompt = create_transaction_prompt(query);
    trace!("📝 Generated prompt for LLM");
    
    let response = llm_client.generate_response(&prompt).await;
    
    match response.success {
        true => {
            debug!("📥 Received LLM response for flow parsing");
            trace!("🤖 LLM response: {}", response.response);
            
            // Extract JSON from response
            let json_str = extract_json_from_response(&response.response)?;
            trace!("📄 Extracted JSON: {}", json_str);
            
            // Parse JSON into TransactionFlow
            let flow: TransactionFlow = serde_json::from_str(&json_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse transaction flow JSON: {}", e))?;
            
            debug!("✅ Successfully parsed transaction flow with {} steps", flow.pipeline.len());
            info!("🎯 Intent: {}", flow.intent);
            
            Ok(flow)
        }
        false => {
            error!("❌ LLM request failed for flow parsing: {:?}", response.error);
            Err(anyhow::anyhow!("LLM request failed: {:?}", response.error))
        }
    }
}

/// Create the transaction flow parsing prompt
fn create_transaction_prompt(query: &str) -> String {
    format!(r#"
You are a transaction flow analyzer for a multi-agent financial system. Parse user queries into executable transaction pipelines. Respond with JSON NOT in a code block.

IMPORTANT: All amounts use {} decimal places internally. When parsing amounts, preserve the exact decimal format provided by the user.

TOOL: "transaction_flow"

INPUT: "{}"

=== TRANSACTION FLOW FORMAT ===
{{
  "tool": "transaction_flow",
  "intent": "semantic description of user's financial goal",
  "pipeline": [
    {{
      "step": 1,
      "action": "swap|ping|check_balance",
      "from_asset": "asset_symbol",
      "to_asset": "asset_symbol",
      "amount": "decimal_number or 'previous_output' or 'user_specified'",
      "semantic_hook": "detailed description for semantic matching",
      "method_pattern": "expected_method_name_pattern"
    }}
  ],
  "expected_outcome": "description of final result"
}}

=== TRANSACTION PARSING RULES ===

**MULTI-HOP DETECTION** - Extract all sequential operations:
- "swap USDT to NEAR then to BTC" → USDT→NEAR→BTC
- "convert ETH to USDT then to ALPHA" → ETH→USDT→ALPHA
- "ping then swap 100 USDT to BTC" → ping → USDT→BTC

**ASSET MAPPING** - Map common aliases:
- USDT, Tether, stablecoin → USDT
- BTC, Bitcoin → BTC
- ETH, Ethereum → ETH
- NEAR, Near Protocol → NEAR
- ALPHA → ALPHA

**SEMANTIC HOOKS** - Create rich descriptions for embedding matching:
- Focus on conversion intent and purpose
- Include financial context and reasoning
- Use natural language for semantic search
- Mention intermediate and final goals

**METHOD PATTERNS** - Generate expected method names:
- swap_X_to_Y → "swap_{{from}}_{{to}}"
- ping → "ping"
- balance → "check_balance"

**AMOUNT HANDLING**:
- Explicit amounts: "swap 1000 USDT" → amount: "1000.00000000"
- Explicit decimals: "swap 1000.5 USDT" → amount: "1000.50000000"
- Implicit amounts: "swap USDT to BTC" → amount: "user_specified"
- Chained amounts: "then to BTC" → amount: "previous_output"
- ALWAYS preserve decimal precision: "0.5" → "0.50000000", "1.23456789" → "1.23456789"

EXAMPLES:

Input: "I want to swap 500 USDT to NEAR then to BTC"
{{
  "tool": "transaction_flow",
  "intent": "User wants to convert 500 USDT stablecoin to NEAR tokens, then convert the resulting NEAR to Bitcoin",
  "pipeline": [
    {{
      "step": 1,
      "action": "swap",
      "from_asset": "USDT",
      "to_asset": "NEAR",
      "amount": "500.00000000",
      "semantic_hook": "convert 500.00000000 USDT stablecoin to NEAR cryptocurrency tokens",
      "method_pattern": "swap_usdt_to_near"
    }},
    {{
      "step": 2,
      "action": "swap",
      "from_asset": "NEAR",
      "to_asset": "BTC",
      "amount": "previous_output",
      "semantic_hook": "exchange NEAR tokens for Bitcoin cryptocurrency",
      "method_pattern": "swap_near_to_btc"
    }}
  ],
  "expected_outcome": "500.00000000 USDT converted to NEAR then to BTC"
}}

Input: "swap 0.5 USDT to BTC"
{{
  "tool": "transaction_flow",
  "intent": "User wants to convert 0.5 USDT stablecoin to Bitcoin cryptocurrency",
  "pipeline": [
    {{
      "step": 1,
      "action": "swap",
      "from_asset": "USDT",
      "to_asset": "BTC",
      "amount": "0.50000000",
      "semantic_hook": "convert 0.50000000 USDT stablecoin to Bitcoin cryptocurrency",
      "method_pattern": "swap_usdt_to_btc"
    }}
  ],
  "expected_outcome": "0.50000000 USDT converted to BTC"
}}

Input: "ping the system"
{{
  "tool": "transaction_flow",
  "intent": "User wants to test system connectivity with a ping operation",
  "pipeline": [
    {{
      "step": 1,
      "action": "ping",
      "from_asset": null,
      "to_asset": null,
      "amount": "none",
      "semantic_hook": "system health check and connectivity test",
      "method_pattern": "ping"
    }}
  ],
  "expected_outcome": "system ping response confirming connectivity"
}}

RULES:
- Extract ALL sequential operations in order
- Create semantic hooks optimized for embedding search
- Generate method patterns matching the codebase
- Handle amount propagation through chains
- Be explicit about asset conversions
- Use financial terminology for semantic richness
- PRESERVE DECIMAL PRECISION: Always format amounts with {} decimal places
- Respond with JSON only, no code blocks"#, DECIMAL_PLACES, query, DECIMAL_PLACES)
}

/// Extract JSON from LLM response (handles potential markdown code blocks)
fn extract_json_from_response(response: &str) -> Result<String, anyhow::Error> {
    trace!("🔍 Extracting JSON from LLM response");
    
    // Look for JSON in code blocks first
    if let Some(start) = response.find("```json") {
        let start = start + 7; // Skip "```json"
        if let Some(end) = response[start..].find("```") {
            let json_str = response[start..start + end].trim();
            debug!("📄 Found JSON in code block");
            return Ok(json_str.to_string());
        }
    }
    
    // Look for JSON between { and }
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            let json_str = response[start..=end].trim();
            debug!("📄 Found JSON between braces");
            return Ok(json_str.to_string());
        }
    }
    
    // If no JSON found, return the whole response
    warn!("⚠️  Could not find JSON delimiters, using full response");
    Ok(response.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_response() {
        let response_with_code_block = r#"
        Here's the transaction flow:
        ```json
        {"tool": "transaction_flow", "intent": "test"}
        ```
        "#;
        
        let result = extract_json_from_response(response_with_code_block).unwrap();
        assert_eq!(result, "{\"tool\": \"transaction_flow\", \"intent\": \"test\"}");
        
        let response_direct = r#"{"tool": "transaction_flow", "intent": "test"}"#;
        let result = extract_json_from_response(response_direct).unwrap();
        assert_eq!(result, "{\"tool\": \"transaction_flow\", \"intent\": \"test\"}");
    }
}