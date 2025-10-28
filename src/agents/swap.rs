use std::sync::Arc;
use fluent_uri::Uri;
use crate::embedding::Embedding;
use crate::hnsw::HnswMemoryIndex;
use crate::ledger::Ledger;
use crate::models::{Channel, CreateAgent, CreateMethod, CreatePort, ExecKind, ProgramAbi, ProgramRef, Visibility};
use crate::queries::Queries;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionRequest {
    pub amount: u128,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Converted {
    pub received_amount: u128,
}

// Add this variant if not present
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConversionError {
    InvalidRate,
    InsufficientFunds { have: u128, required: u128 },
}

pub type ConversionResult = Result<Converted, ConversionError>;

pub struct Swap;

impl Swap {
    pub async fn create_agent(queries: &Queries, embedding: Arc<Embedding>, mut hnsw: HnswMemoryIndex<'_>) -> Result<(), anyhow::Error> {
        // 1) Create the parent "Swap Agent"
        let create_agent = CreateAgent {
            label: "SwapAgent".to_string(),
            version: "1.0.0".to_string(),
            methods: vec![], // filled by separate CreateMethod calls
            description: "Performs token-to-token swaps across supported assets.".to_string(),
        };
        let agent_handle = queries.create_agent(create_agent).await?;

        // 2) Define supported assets and type URIs
        let assets = ["ALPHA", "USDT", "BTC", "ETH", "NEAR"];
        let conv_req_type: Uri<String> = "type://SwapAgent/ConversionRequest@1".parse()?;
        let asset_amt_type: Uri<String> = "type://SwapAgent/AssetAmount@1".parse()?;

        let mut method_handles = vec![];
        // 3) Generate one Method for every ordered pair FROM != TO
        for &from in &assets {
            for &to in &assets {
                if from == to { continue; }

                // Human-friendly labels and descriptions
                let label = format!("Swap {from}→{to}");
                let description = format!(
                    "Swap assets from {from} to {to}. Expects {{\"amount\"}}. \
                     emits an AssetAmount for {to}."
                );

                // Per-method embedding (good for discovery by NL intent)
                let embedding = embedding.embed(&description).await?;

                // Endpoints (unique, but simple and predictable)
                let in_ep  = Uri::parse(format!("agent://SwapAgent/swap#{from}_{to}_in"))?.to_owned();
                let out_ep = Uri::parse(format!("agent://SwapAgent/swap#{from}_{to}_out"))?.to_owned();

                // LocalFn export name you can register in your executor registry
                let export = format!("swap_{}_{}", from, to);

                // 4) Build and insert the method
                let method = CreateMethod {
                    agent: agent_handle.id.clone(),       // FK to parent
                    label,
                    version: "1.0.0".to_string(),
                    visibility: Visibility::Public,
                    exec_kind: ExecKind::Local,           // demo via LocalFn
                    description,
                    in_port: CreatePort {
                        label: format!("in.swap-{from}-{to}"),
                        description: format!("Accepts a ConversionRequest from {from} to {to}."),
                        channel: Channel::Call,
                        type_uri: Some(conv_req_type.clone()),
                        end_point: in_ep,
                    },
                    out_port: CreatePort {
                        label: format!("out.swap-{from}-{to}"),
                        description: format!("Emits AssetAmount of {to}."),
                        channel: Channel::Call,
                        type_uri: Some(asset_amt_type.clone()),
                        end_point: out_ep,
                    },
                    program: ProgramRef {
                        module_uri: Uri::parse(format!("local://SwapAgent/swap_{from}_{to}"))?.to_owned(),
                        export,                           // e.g., swap_ALPHA_USDT
                        abi: ProgramAbi::LocalFn,
                        checksum: "demo".to_string(),     // replace with real blake3/sha256 if you like
                    },
                    embedding: embedding.clone(),
                };

                let method_handle = queries.create_method(method).await?;
                hnsw.add(method_handle.id.clone(), &embedding)?;
                method_handles.push(method_handle);
            }
        }

        Ok(())
    }
}

/// Generate one swap execution function per pair, with integer (num/den) rates.
/// Now you explicitly name each generated function per pair.
#[macro_export]
macro_rules! define_swaps {
    (
        ledger_ty = $ledger_ty:ty;
        $(
            ($from_ident:ident, $from_sym:literal) => {
                $(
                    fn $fn_name:ident ($to_ident:ident, $to_sym:literal) : { num: $num:expr, den: $den:expr }
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        $(
            $(
                #[allow(non_snake_case)]
                pub fn $fn_name(
                    ledger: &mut $ledger_ty,
                    request: ConversionRequest,
                ) -> ConversionResult {
                    // Guard: denominator must be > 0
                    if ($den) == 0 {
                        return Err(ConversionError::InvalidRate);
                    }
                    let from_symbol: &str = $from_sym;
                    let to_symbol: &str = $to_sym;
                    let amount = request.amount;

                    // Balance check
                    let have = ledger.get_balance(from_symbol);
                    if have < amount {
                        return Err(ConversionError::InsufficientFunds { have, required: amount });
                    }

                    // Debit source
                    ledger.debit(from_symbol, amount);

                    // Integer math: received = amount * num / den
                    let received = amount
                        .saturating_mul(($num as u128))
                        .checked_div(($den as u128))
                        .unwrap_or(0);

                    // Credit destination
                    ledger.credit(to_symbol, received);

                    Ok(Converted { received_amount: received })
                }
            )*
        )*
    };
}

define_swaps! {
    ledger_ty = Ledger;

    (alpha, "ALPHA") => {
        fn swap_alpha_to_usdt (usdt, "USDT"): { num: 300u128,     den: 100u128     },
        fn swap_alpha_to_btc  (btc,  "BTC") : { num: 300u128,     den: 11476500u128 },
        fn swap_alpha_to_eth  (eth,  "ETH") : { num: 300u128,     den: 411438u128  },
        fn swap_alpha_to_near (near, "NEAR"): { num: 300u128,     den: 230u128     },
    },

    (usdt, "USDT") => {
        fn swap_usdt_to_alpha (alpha, "ALPHA"): { num: 100u128,      den: 300u128    },
        fn swap_usdt_to_btc   (btc,   "BTC")  : { num: 100u128,      den: 11476500u128 },
        fn swap_usdt_to_eth   (eth,   "ETH")  : { num: 100u128,      den: 411438u128 },
        fn swap_usdt_to_near  (near,  "NEAR") : { num: 100u128,      den: 230u128    },
    },

    (btc, "BTC") => {
        fn swap_btc_to_usdt  (usdt, "USDT"): { num: 11476500u128, den: 100u128    },
        fn swap_btc_to_eth   (eth,  "ETH") : { num: 11476500u128, den: 411438u128 },
        fn swap_btc_to_alpha (alpha,"ALPHA"): { num: 11476500u128, den: 300u128   },
        fn swap_btc_to_near  (near, "NEAR"): { num: 11476500u128, den: 230u128   },
    },

    (eth, "ETH") => {
        fn swap_eth_to_usdt  (usdt, "USDT"): { num: 411438u128,   den: 100u128    },
        fn swap_eth_to_btc   (btc,  "BTC") : { num: 411438u128,   den: 11476500u128 },
        fn swap_eth_to_alpha (alpha,"ALPHA"): { num: 411438u128,   den: 300u128    },
        fn swap_eth_to_near  (near, "NEAR"): { num: 411438u128,   den: 230u128    },
    },

    (near, "NEAR") => {
        fn swap_near_to_usdt (usdt, "USDT"): { num: 230u128,      den: 100u128    },
        fn swap_near_to_alpha(alpha,"ALPHA"): { num: 230u128,      den: 300u128    },
        fn swap_near_to_eth  (eth,  "ETH") : { num: 230u128,      den: 411438u128 },
        fn swap_near_to_btc  (btc,  "BTC") : { num: 230u128,      den: 11476500u128 },
    },
}
