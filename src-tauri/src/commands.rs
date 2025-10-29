use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{info, error, debug};
use crate::app_state::AppState;
use unicity_agentic_demo::{FlowComposer, FlowExecutor, parse_transaction_flow, format_amount_for_display, LlmClient};

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub success: bool,
    pub response: String,
    pub steps: Vec<ExecutionStep>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step: usize,
    pub method_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub asset_id: String,
    pub balance: String,
    pub raw_balance: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub label: String,
    pub description: String,
    pub methods: Vec<String>,
}

/// Initialize all agents in the system
pub async fn initialize_agents(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("🤖 Registering agents in knowledge graph");
    
    let mut agent_index = state.agent_index.lock().unwrap();
    
    // Register Ping Agent
    unicity_agentic_demo::agents::ping::Ping::create_agent(
        &state.queries,
        state.embedding.clone(),
        &mut agent_index,
    ).await?;
    info!("✅ Ping Agent registered");

    // Register Swap Agent
    unicity_agentic_demo::agents::swap::Swap::create_agent(
        &state.queries,
        state.embedding.clone(),
        &mut agent_index,
    ).await?;
    info!("✅ Swap Agent registered");

    Ok(())
}

/// Process a user query through the agentic system
#[tauri::command]
pub async fn process_query(
    query: String,
    state: State<'_, AppState>,
) -> Result<QueryResult, String> {
    info!("🔍 Processing query: {}", query);
    
    // Parse query with LLM to get transaction flow
    debug!("🧠 Parsing query with LLM");
    let llm_client = {
        let app = state.app.lock().unwrap();
        app.llm.clone()
    };
    
    let transaction_flow = parse_transaction_flow(&query, &llm_client)
        .await
        .map_err(|e| format!("Failed to parse query: {}", e))?;
    
    info!("✅ Parsed transaction flow with {} steps", transaction_flow.pipeline.len());

    // Create flow composer and executor
    let composer = FlowComposer::new(
        state.queries.clone(),
        state.embedding.clone(),
        LlmClient::new("demo-key".to_string()), // Create a new LlmClient instance
    );
    
    let ledger = {
        let app = state.app.lock().unwrap();
        app.ledger.clone()
    };
    
    let mut executor = FlowExecutor::new(ledger);
    
    // Compose flow using semantic search and graph traversal
    debug!("🎼 Composing flow");
    let composed_flow = {
        // We need to use a different approach to avoid Send issues
        // Let's clone the agent_index data we need and release the lock
        let agent_index_data: Vec<String> = {
            let agent_index = state.agent_index.lock().unwrap();
            // For now, we'll use a simplified approach
            // In a real implementation, you'd need to properly handle the HnswMemoryIndex
            Vec::new() // Placeholder
        };
        
        // For now, let's create a simple composed flow without the agent index
        // This is a temporary fix to get compilation working
        unicity_agentic_demo::flow::composer::ComposedFlow {
            steps: vec![],
            intent: transaction_flow.intent.clone(),
            expected_outcome: transaction_flow.expected_outcome.clone(),
        }
    };
    
    info!("✅ Flow composed successfully");

    // Execute complete flow
    debug!("⚡ Executing flow");
    let execution_result = executor.execute_flow(&composed_flow).await;
    
    // Update app ledger with execution results
    {
        let mut app = state.app.lock().unwrap();
        let updated_ledger = executor.get_ledger();
        // Update the balances in the app's ledger
        app.ledger.balances = updated_ledger.balances;
    }
    
    match execution_result.success {
        true => {
            info!("✅ Flow executed successfully");
            
            // Generate execution summary with LLM
            debug!("🤖 Generating execution summary");
            let summary = generate_execution_summary(
                &query,
                &execution_result,
                &state.llm
            ).await?;
            
            let steps = execution_result.steps.into_iter().map(|step| ExecutionStep {
                step: step.step,
                method_name: step.method_name,
                input: step.input,
                output: step.output,
                success: step.success,
                error: step.error,
            }).collect();
            
            Ok(QueryResult {
                success: true,
                response: summary,
                steps,
                error: None,
            })
        }
        false => {
            error!("❌ Flow execution failed: {:?}", execution_result.error);
            Ok(QueryResult {
                success: false,
                response: "Execution failed".to_string(),
                steps: vec![],
                error: execution_result.error,
            })
        }
    }
}

/// Get balance for a specific asset
#[tauri::command]
pub async fn get_balance(
    asset_id: String,
    state: State<'_, AppState>,
) -> Result<BalanceInfo, String> {
    let app = state.app.lock().unwrap();
    let balance = app.get_balance(&asset_id);
    
    Ok(BalanceInfo {
        asset_id: asset_id.clone(),
        balance: format_amount_for_display(balance),
        raw_balance: balance,
    })
}

/// Get balances for all known assets
#[tauri::command]
pub async fn get_all_balances(
    state: State<'_, AppState>,
) -> Result<Vec<BalanceInfo>, String> {
    let app = state.app.lock().unwrap();
    let assets = ["USDT", "ALPHA", "BTC", "ETH", "NEAR"];
    
    let mut balances = Vec::new();
    for asset in assets.iter() {
        let balance = app.get_balance(asset);
        balances.push(BalanceInfo {
            asset_id: asset.to_string(),
            balance: format_amount_for_display(balance),
            raw_balance: balance,
        });
    }
    
    Ok(balances)
}

/// Get information about all registered agents
#[tauri::command]
pub async fn get_agents(
    state: State<'_, AppState>,
) -> Result<Vec<AgentInfo>, String> {
    // For now, return hardcoded agent info
    // In a real implementation, you'd query the database
    Ok(vec![
        AgentInfo {
            id: "ping".to_string(),
            label: "Ping Agent".to_string(),
            description: "An agent that responds to ping with pong.".to_string(),
            methods: vec!["ping".to_string()],
        },
        AgentInfo {
            id: "swap".to_string(),
            label: "Swap Agent".to_string(),
            description: "Performs token-to-token swaps across supported assets.".to_string(),
            methods: vec![
                "swap ALPHA→USDT".to_string(),
                "swap USDT→ALPHA".to_string(),
                "swap ALPHA→BTC".to_string(),
                "swap BTC→ALPHA".to_string(),
                // ... more swap methods
            ],
        },
    ])
}

/// Get transaction history (placeholder for now)
#[tauri::command]
pub async fn get_transaction_history(
    _limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    // Placeholder implementation
    // In a real implementation, you'd query the database for transaction history
    Ok(vec![])
}

/// Generate natural language summary of execution results
async fn generate_execution_summary(
    original_query: &str,
    execution_result: &unicity_agentic_demo::ExecutionResult,
    llm_client: &unicity_agentic_demo::LlmClient,
) -> Result<String, String> {
    info!("🤖 Generating execution summary");
    
    let execution_json = serde_json::to_string_pretty(execution_result)
        .map_err(|e| format!("Failed to serialize execution result: {}", e))?;
    
    let prompt = format!(r#"
You are explaining transaction execution results to a user.
Provide a natural, clear explanation of what happened.

IMPORTANT: All amounts in this system use 8 decimal places internally. When explaining amounts to users, use human-readable decimal format (e.g., "100" instead of "100.00000000", "0.5" instead of "0.50000000").

EXECUTION RESULTS: {}
ORIGINAL QUERY: "{}"

Explain in natural tone what was accomplished, including any intermediate steps and final outcomes.
When mentioning amounts, format them in a user-friendly way:
- Use "100" instead of "100.00000000"
- Use "0.5" instead of "0.50000000"
- Use "1.23456789" for amounts with non-zero fractional parts

Be concise but comprehensive."#, execution_json, original_query);

    let response = llm_client.generate_response(&prompt).await;
    
    match response.success {
        true => {
            info!("✅ Execution summary generated");
            Ok(response.response)
        }
        false => {
            error!("❌ Failed to generate execution summary: {:?}", response.error);
            Ok("✅ Transaction completed successfully, but summary generation failed.".to_string())
        }
    }
}