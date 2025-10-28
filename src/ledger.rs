use tracing::{trace, debug, warn};
use crate::decimal::format_amount_for_display;

pub type AssetId = String;

// Note for example, we are using 8 decimals for everything.
#[derive(Debug, Clone)]
pub struct Ledger {
    pub balances: Vec<(AssetId, u128)>,
}

impl Ledger {
    pub fn new() -> Self {
        trace!("💰 Creating new ledger");
        let ledger = Ledger {
            balances: Vec::new(),
        };
        debug!("💰 New ledger created with empty balances");
        ledger
    }

    pub fn get_balance(&self, asset_id: &str) -> u128 {
        trace!("💰 Querying balance for asset: {}", asset_id);
        for (id, balance) in &self.balances {
            if id == asset_id {
                debug!("💰 Found balance for {}: {} (internal: {})", asset_id, format_amount_for_display(*balance), balance);
                return *balance;
            }
        }
        debug!("💰 No balance found for {}, returning 0", asset_id);
        0
    }

    pub fn set_balance(&mut self, asset_id: AssetId, amount: u128) {
        trace!("💰 Setting balance for {}: {} (internal: {})", asset_id, format_amount_for_display(amount), amount);
        for (id, balance) in &mut self.balances {
            if id == &asset_id {
                debug!("💰 Updating existing balance for {}: {} -> {} (internal: {} -> {})",
                    asset_id, format_amount_for_display(*balance), format_amount_for_display(amount), balance, amount);
                *balance = amount;
                return;
            }
        }
        debug!("💰 Adding new balance entry for {}: {} (internal: {})", asset_id, format_amount_for_display(amount), amount);
        self.balances.push((asset_id, amount));
    }

    pub fn debit(&mut self, asset_id: &str, amount: u128) {
        trace!("💳 Debiting {} from asset: {} (internal: {})", format_amount_for_display(amount), asset_id, amount);
        for (id, balance) in &mut self.balances {
            if id == asset_id {
                if *balance >= amount {
                    let new_balance = *balance - amount;
                    debug!("💳 Debiting {} from {}: {} -> {} (internal: {} -> {} -> {})",
                        format_amount_for_display(amount), asset_id, format_amount_for_display(*balance),
                        format_amount_for_display(new_balance), balance, amount, new_balance);
                    *balance = new_balance;
                } else {
                    warn!("⚠️  Insufficient balance for debit {}: have {}, need {} (internal: {} -> {})",
                        asset_id, format_amount_for_display(*balance), format_amount_for_display(amount), balance, amount);
                    debug!("💳 Setting balance to 0 for {} (insufficient funds)", asset_id);
                    *balance = 0;
                }
                return;
            }
        }
        warn!("⚠️  Attempted to debit from non-existent asset: {}", asset_id);
    }

    pub fn credit(&mut self, asset_id: &str, amount: u128) {
        trace!("💳 Crediting {} to asset: {} (internal: {})", format_amount_for_display(amount), asset_id, amount);
        for (id, balance) in &mut self.balances {
            if id == asset_id {
                let new_balance = *balance + amount;
                debug!("💳 Crediting {} to {}: {} -> {} (internal: {} -> {} -> {})",
                    format_amount_for_display(amount), asset_id, format_amount_for_display(*balance),
                    format_amount_for_display(new_balance), balance, amount, new_balance);
                *balance = new_balance;
                return;
            }
        }
        debug!("💳 Adding new balance entry for {}: {} (internal: {})", asset_id, format_amount_for_display(amount), amount);
        self.balances.push((asset_id.to_string(), amount));
    }
}
