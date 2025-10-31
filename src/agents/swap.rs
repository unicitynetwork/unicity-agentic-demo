use crate::embedding::Embedding;
use crate::hnsw::HnswMemoryIndex;
use crate::ledger::Ledger;
use crate::models::{
    Channel, CreateAgent, CreateMethod, CreatePort, ExecKind, ProgramAbi, ProgramRef, Visibility,
};
use crate::queries::Queries;
use fluent_uri::Uri;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionRequest {
    pub amount: u128,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Converted {
    pub received_amount: u128,
}

// TODO: Like logs, errors should be on a different outpoint.
// // Add this variant if not present
// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub enum ConversionError {
//     InvalidRate,
//     InsufficientFunds { have: u128, required: u128 },
// }
//
// pub type ConversionResult = Result<Converted, ConversionError>;

pub struct Swap;

impl Swap {
    pub async fn create_agent(
        queries: &Queries,
        embedding: Arc<Embedding>,
        hnsw: &mut HnswMemoryIndex<'_>,
    ) -> Result<(), anyhow::Error> {
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
        let conv_req_type: Uri<String> = "type://u128".parse()?;
        let asset_amt_type: Uri<String> = "type://u128".parse()?;

        let mut method_handles = vec![];
        // 3) Generate one Method for every ordered pair FROM != TO
        for &from in &assets {
            for &to in &assets {
                if from == to {
                    continue;
                }

                // Human-friendly labels and descriptions
                let label = format!("Swap {from}→{to}");
                let description = format!(
                    "Swap assets from {from} to {to}. Expects {{\"amount\"}}. \
                     emits an AssetAmount for {to}."
                );

                // Per-method embedding (good for discovery by NL intent)
                let embedding = embedding.embed(&description).await?;

                // Endpoints (unique, but simple and predictable)
                let in_ep =
                    Uri::parse(format!("agent://SwapAgent/swap#{from}_{to}_in"))?.to_owned();
                let out_ep =
                    Uri::parse(format!("agent://SwapAgent/swap#{from}_{to}_out"))?.to_owned();

                // LocalFn export name you can register in your executor registry
                let export = format!("swap_{}_to_{}", from, to).to_lowercase();

                // 4) Build and insert the method
                let method = CreateMethod {
                    agent: agent_handle.id.clone(), // FK to parent
                    label,
                    version: "1.0.0".to_string(),
                    visibility: Visibility::Public,
                    exec_kind: ExecKind::Local, // demo via LocalFn
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
                        module_uri: Uri::parse(format!("local://SwapAgent/swap_{from}_{to}"))?
                            .to_owned(),
                        export, // e.g., swap_ALPHA_USDT
                        abi: ProgramAbi::LocalFn,
                        checksum: "demo".to_string(), // replace with real blake3/sha256 if you like
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
                    request: u128,
                ) -> u128 {
                    // Guard: denominator must be > 0
                    if ($den) == 0 {
                        return 0;
                    }
                    let from_symbol: &str = $from_sym;
                    let to_symbol: &str = $to_sym;
                    let amount = request;

                    // Balance check
                    let have = ledger.get_balance(from_symbol);
                    if have < amount {
                        return 0;
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

                    received
                }
            )*
        )*
    };
}

define_swaps! {
    ledger_ty = Ledger;

    (alpha, "ALPHA") => {
        fn swap_alpha_to_usdt (usdt, "USDT"): { num: 300_000_000u128,        den: 100_000_000u128        }, // 1 ALPHA = 3.00000000 USDT
        fn swap_alpha_to_btc  (btc,  "BTC") : { num: 300_000_000u128,        den: 11_476_500_000_000u128 }, // ≈ 0.00002614037380 BTC
        fn swap_alpha_to_eth  (eth,  "ETH") : { num: 300_000_000u128,        den: 411_438_000_000u128    }, // ≈ 0.00072914995698 ETH
        fn swap_alpha_to_near (near, "NEAR"): { num: 300_000_000u128,        den: 230_000_000u128        }, // ≈ 1.30434782609 NEAR
    },

    (usdt, "USDT") => {
        fn swap_usdt_to_alpha (alpha, "ALPHA"): { num: 100_000_000u128,        den: 300_000_000u128       }, // ≈ 0.33333333333 ALPHA
        fn swap_usdt_to_btc   (btc,   "BTC")  : { num: 100_000_000u128,        den: 11_476_500_000_000u128}, // ≈ 0.00000871345794 BTC
        fn swap_usdt_to_eth   (eth,   "ETH")  : { num: 100_000_000u128,        den: 411_438_000_000u128   }, // ≈ 0.00024304998566 ETH
        fn swap_usdt_to_near  (near,  "NEAR") : { num: 100_000_000u128,        den: 230_000_000u128       }, // ≈ 0.43478260870 NEAR
    },

    (btc, "BTC") => {
        fn swap_btc_to_usdt  (usdt, "USDT"): { num: 11_476_500_000_000u128, den: 100_000_000u128        }, // 1 BTC = 114,765.00000000 USDT
        fn swap_btc_to_eth   (eth,  "ETH") : { num: 11_476_500_000_000u128, den: 411_438_000_000u128    }, // ≈ 27.89363160428 ETH
        fn swap_btc_to_alpha (alpha,"ALPHA"): { num: 11_476_500_000_000u128, den: 300_000_000u128       }, // ≈ 38,255.00000000 ALPHA
        fn swap_btc_to_near  (near, "NEAR"): { num: 11_476_500_000_000u128, den: 230_000_000u128       }, // ≈ 49,897.82608696 NEAR
    },

    (eth, "ETH") => {
        fn swap_eth_to_usdt  (usdt, "USDT"): { num: 411_438_000_000u128,    den: 100_000_000u128        }, // 1 ETH = 4,114.38000000 USDT
        fn swap_eth_to_btc   (btc,  "BTC") : { num: 411_438_000_000u128,    den: 11_476_500_000_000u128 }, // ≈ 0.03585047706 BTC
        fn swap_eth_to_alpha (alpha,"ALPHA"): { num: 411_438_000_000u128,    den: 300_000_000u128       }, // ≈ 1,371.46000000 ALPHA
        fn swap_eth_to_near  (near, "NEAR"): { num: 411_438_000_000u128,    den: 230_000_000u128       }, // ≈ 1,788.86086957 NEAR
    },

    (near, "NEAR") => {
        fn swap_near_to_usdt (usdt, "USDT"): { num: 230_000_000u128,        den: 100_000_000u128        }, // 1 NEAR = 2.30000000 USDT
        fn swap_near_to_alpha(alpha,"ALPHA"): { num: 230_000_000u128,        den: 300_000_000u128       }, // ≈ 0.76666666667 ALPHA
        fn swap_near_to_eth  (eth,  "ETH") : { num: 230_000_000u128,        den: 411_438_000_000u128   }, // ≈ 0.00055901497 ETH
        fn swap_near_to_btc  (btc,  "BTC") : { num: 230_000_000u128,        den: 11_476_500_000_000u128}, // ≈ 0.00002004095 BTC
    },
}
