pub type AssetId = String;

// Note for example, we are using 2 decimals for everything.
#[derive(Debug, Clone)]
pub struct Ledger {
    balances: Vec<(AssetId, u128)>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            balances: Vec::new(),
        }
    }

    pub fn get_balance(&self, asset_id: &str) -> u128 {
        for (id, balance) in &self.balances {
            if id == asset_id {
                return *balance;
            }
        }
        0
    }

    pub fn set_balance(&mut self, asset_id: AssetId, amount: u128) {
        for (id, balance) in &mut self.balances {
            if id == &asset_id {
                *balance = amount;
                return;
            }
        }
        self.balances.push((asset_id, amount));
    }

    pub fn debit(&mut self, asset_id: &str, amount: u128) {
        for (id, balance) in &mut self.balances {
            if id == asset_id {
                if *balance >= amount {
                    *balance -= amount;
                } else {
                    *balance = 0;
                }
                return;
            }
        }
    }

    pub fn credit(&mut self, asset_id: &str, amount: u128) {
        for (id, balance) in &mut self.balances {
            if id == asset_id {
                *balance += amount;
                return;
            }
        }
        self.balances.push((asset_id.to_string(), amount));
    }
}
