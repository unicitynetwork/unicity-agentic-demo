mod models;
mod app;
mod ledger;
mod queries;
mod executor;
pub mod agents;
mod embedding;
mod llm;
mod hnsw;
mod flow;
mod decimal;

// Re-export public API
pub use app::App;
pub use queries::Queries;
pub use embedding::Embedding;
pub use llm::LlmClient;
pub use hnsw::HnswMemoryIndex;
pub use ledger::Ledger;
pub use flow::{TransactionFlow, FlowStep, ExecutionResult, FlowComposer, FlowExecutor, parse_transaction_flow};
pub use decimal::{parse_decimal_amount, format_decimal_amount, format_amount_for_llm, parse_amount_from_llm, format_amount_for_display, DECIMAL_PLACES, DECIMAL_FACTOR};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
