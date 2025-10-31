use crate::app_state::AppState;
use crate::constants::DEFAULT_TRANSACTION_HISTORY_LIMIT;
use crate::stt::SpeechRecognizer;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};
use unicity_agentic_demo::{
    format_amount_for_display, parse_transaction_flow, FlowComposer, FlowExecutor,
};

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
    // pub methods: Vec<String>,
}

/// Initialize all agents in the system
pub async fn initialize_agents(state: &AppState) -> anyhow::Result<()> {
    info!("🤖 Registering agents in knowledge graph");

    let mut agent_index = state
        .agent_index
        .lock()
        .map_err(|_| anyhow::Error::msg("Couldn't get lock on Memory Index"))?;

    // Register Ping Agent
    unicity_agentic_demo::agents::ping::Ping::create_agent(
        &state.queries,
        state.embedding.clone(),
        &mut agent_index,
    )
    .await?;
    info!("✅ Ping Agent registered");

    // Register Swap Agent
    unicity_agentic_demo::agents::swap::Swap::create_agent(
        &state.queries,
        state.embedding.clone(),
        &mut agent_index,
    )
    .await?;
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

    info!(
        "✅ Parsed transaction flow with {} steps",
        transaction_flow.pipeline.len()
    );

    // Create flow composer and executor
    let _composer = FlowComposer::new(
        state.queries.clone(),
        state.embedding.clone(),
        (*state.llm).clone(),
    );

    let ledger = {
        let app = state.app.lock().unwrap();
        app.ledger.clone()
    };

    let mut executor = FlowExecutor::new(ledger);

    // Compose flow using semantic search and graph traversal
    debug!("🎼 Composing flow");

    // We need to handle the HNSW index carefully since it's not Send
    // The solution is to extract all the data we need from the HNSW index first,
    // then release the lock before doing async operations

    // First, get the embedding for each step's semantic hook
    let mut step_embeddings = Vec::new();
    for step in &transaction_flow.pipeline {
        let embedding = state
            .embedding
            .embed(&step.semantic_hook)
            .await
            .map_err(|e| format!("Failed to generate embedding: {}", e))?;
        step_embeddings.push(embedding);
    }

    // Now search the HNSW index for each step (this is synchronous)
    let mut search_results = Vec::new();
    {
        let agent_index = state.agent_index.lock().unwrap();
        for (embedding, _step) in step_embeddings.iter().zip(transaction_flow.pipeline.iter()) {
            let results = agent_index.search(embedding, 10);
            search_results.push(results);
        }
    }

    // Now create the composer and compose the flow using the pre-computed search results
    let composed_flow = _composer
        .compose_flow_with_search_results(&transaction_flow, &search_results)
        .await
        .map_err(|e| format!("Failed to compose flow: {}", e))?;

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
            let summary =
                generate_execution_summary(&query, &execution_result, &(*state.llm).clone())
                    .await
                    .map_err(|e| e.to_string())?;

            let steps = execution_result
                .steps
                .into_iter()
                .map(|step| ExecutionStep {
                    step: step.step,
                    method_name: step.method_name,
                    input: step.input,
                    output: step.output,
                    success: step.success,
                    error: step.error,
                })
                .collect();

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
pub async fn get_all_balances(state: State<'_, AppState>) -> Result<Vec<BalanceInfo>, String> {
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
pub async fn get_agents(state: State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    // Query the database for registered agents
    let agents = state
        .queries
        .get_all_agents()
        .await
        .map_err(|e| format!("Failed to query agents: {}", e))?;

    let mut agent_infos = Vec::new();
    for agent in agents {
        // let methods = state.queries.get_agent_methods(&agent.id)
        //     .await
        //     .map_err(|e| format!("Failed to query agent methods: {}", e))?;

        agent_infos.push(AgentInfo {
            id: agent.id.to_string(),
            label: agent.label,
            description: agent.description,
            // methods: methods.into_iter().map(|m| m.name).collect(),
        });
    }

    Ok(agent_infos)
}

/// Get transaction history
#[tauri::command]
pub async fn get_transaction_history(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    // Query the database for transaction history
    let transactions = state
        .queries
        .get_transaction_history(limit.unwrap_or(DEFAULT_TRANSACTION_HISTORY_LIMIT))
        .await
        .map_err(|e| format!("Failed to query transaction history: {}", e))?;

    // Convert transactions to JSON values
    let json_transactions = transactions
        .into_iter()
        .map(|tx| {
            serde_json::to_value(tx).map_err(|e| format!("Failed to serialize transaction: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_transactions)
}

/// Start speech recognition
#[tauri::command]
pub async fn stt_start(sr: State<'_, SpeechRecognizer>) -> Result<(), String> {
    info!("🎤 stt_start command called");
    if sr.is_recognizing() {
        info!("🎤 Speech recognition already active");
        return Ok(());
    }
    info!("🎤 Starting speech recognition");
    sr.start_recognition().await;
    Ok(())
}

#[tauri::command]
pub async fn stt_stop(sr: State<'_, SpeechRecognizer>) -> Result<(), String> {
    info!("🛑 stt_stop command called");
    if !sr.is_recognizing() {
        info!("🛑 Speech recognition not active, nothing to stop");
        return Ok(());
    }
    info!("🛑 Stopping speech recognition");
    sr.stop_recognition().await;
    Ok(())
}

/// Generate natural language summary of execution results
async fn generate_execution_summary(
    original_query: &str,
    execution_result: &unicity_agentic_demo::ExecutionResult,
    llm_client: &unicity_agentic_demo::LlmClient,
) -> anyhow::Result<String> {
    info!("🤖 Generating execution summary");

    let execution_json = serde_json::to_string_pretty(execution_result)
        .expect("Failed to serialize execution result");

    let prompt = format!(
        r#"
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

Be concise but comprehensive."#,
        execution_json, original_query
    );

    let response = llm_client.generate_response(&prompt).await;

    match response.success {
        true => {
            info!("✅ Execution summary generated");
            Ok(response.response)
        }
        false => {
            error!(
                "❌ Failed to generate execution summary: {:?}",
                response.error
            );
            Ok("✅ Transaction completed successfully, but summary generation failed.".to_string())
        }
    }
}
