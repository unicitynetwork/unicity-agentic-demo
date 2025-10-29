//! Library interface for unicity-agentic-demo
//! 
//! This module provides the public API for the unicity-agentic-demo library,
//! allowing it to be used as a dependency in the Tauri UI.

pub mod agents;
pub mod app;
pub mod decimal;
pub mod embedding;
pub mod executor;
pub mod flow;
pub mod hnsw;
pub mod ledger;
pub mod llm;
pub mod models;
pub mod queries;

// Re-export public API
pub use app::App;
pub use queries::Queries;
pub use embedding::Embedding;
pub use llm::LlmClient;
pub use hnsw::HnswMemoryIndex;
pub use ledger::Ledger;
pub use flow::{TransactionFlow, FlowStep, ExecutionResult, FlowComposer, FlowExecutor, parse_transaction_flow};
pub use decimal::{parse_decimal_amount, format_decimal_amount, format_amount_for_llm, parse_amount_from_llm, format_amount_for_display, DECIMAL_PLACES, DECIMAL_FACTOR};
