use tracing::{trace, debug, warn};

pub type AssetId = String;

// Note for example, we are using 2 decimals for everything.
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
                debug!("💰 Found balance for {}: {} (2 decimal places)", asset_id, balance);
                return *balance;
            }
        }
        debug!("💰 No balance found for {}, returning 0", asset_id);
        0
    }

    pub fn set_balance(&mut self, asset_id: AssetId, amount: u128) {
        trace!("💰 Setting balance for {}: {}", asset_id, amount);
        for (id, balance) in &mut self.balances {
            if id == &asset_id {
                debug!("💰 Updating existing balance for {}: {} -> {}", asset_id, balance, amount);
                *balance = amount;
                return;
            }
        }
        debug!("💰 Adding new balance entry for {}: {}", asset_id, amount);
        self.balances.push((asset_id, amount));
    }

    pub fn debit(&mut self, asset_id: &str, amount: u128) {
        trace!("💳 Debiting {} from asset: {}", amount, asset_id);
        for (id, balance) in &mut self.balances {
            if id == asset_id {
                if *balance >= amount {
                    debug!("💳 Debiting {} from {}: {} -> {}", amount, asset_id, balance, *balance - amount);
                    *balance -= amount;
                } else {
                    warn!("⚠️  Insufficient balance for debit {}: have {}, need {}", asset_id, balance, amount);
                    debug!("💳 Setting balance to 0 for {} (insufficient funds)", asset_id);
                    *balance = 0;
                }
                return;
            }
        }
        warn!("⚠️  Attempted to debit from non-existent asset: {}", asset_id);
    }

    pub fn credit(&mut self, asset_id: &str, amount: u128) {
        trace!("💳 Crediting {} to asset: {}", amount, asset_id);
        for (id, balance) in &mut self.balances {
            if id == asset_id {
                debug!("💳 Crediting {} to {}: {} -> {}", amount, asset_id, balance, *balance + amount);
                *balance += amount;
                return;
            }
        }
        debug!("💳 Adding new balance entry for {}: {}", asset_id, amount);
        self.balances.push((asset_id.to_string(), amount));
    }
}
