use serde_json::Value;
use crate::ledger::Ledger;

mod ping;
mod swap;

pub trait Agent<I, O, E> {
    fn execute(&self, ledger: &mut Ledger, input: I) -> Result<O, E>;
}
