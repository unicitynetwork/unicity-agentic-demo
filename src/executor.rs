use serde_json::{json, Value};
use crate::agents::{ConversionRequest};
use crate::ledger::Ledger;
use crate::models::Method;
use crate::agents::*;

fn swap_json_adapter<F>(f: F, ledger: &mut Ledger, args: Value) -> Value
where
    F: Fn(&mut Ledger, u128) -> u128
{
    match serde_json::from_value::<ConversionRequest>(args) {
        Ok(req) => {
            let out = f(ledger, req.amount);      // f returns a bare u128 now
            json!(out)                     // return it as a JSON number
        }
        Err(e) => json!({ "error": format!("bad args: {e}") }),
    }
}

pub fn ping(_ledger: &mut Ledger, input: Value) -> Value {
    Value::String(format!("pong: {}", input))
}

macro_rules! swap_dispatch {
    ($export:expr, $ledger:expr, $args:expr, $( $fname:ident ),+ $(,)?) => {
        match $export {
            $(
                stringify!($fname) => swap_json_adapter($fname, $ledger, $args),
            )+
            _ => json!({"ok": false, "error": format!("unknown export: {}", $export)}),
        }
    };
}

/// Unified JSON-in/JSON-out local executor.
/// - If `export == "ping"`, runs `ping`.
/// - If `export` matches a generated `swap_*` function, adapts JSON to `ConversionRequest` and executes.
/// - Returns JSON for success/error.
pub fn exec_local(method: &Method, ledger: &mut Ledger, args: Value) -> Value {
    let export = method.program.export.as_str();
    match export {
        "ping" => ping(ledger, args),
        other if other.starts_with("swap_") => {
            // keep this list in sync with your define_swaps! invocation
            swap_dispatch!(
                other, ledger, args,
                swap_alpha_to_usdt, swap_alpha_to_btc,  swap_alpha_to_eth,  swap_alpha_to_near,
                swap_usdt_to_alpha, swap_usdt_to_btc,   swap_usdt_to_eth,   swap_usdt_to_near,
                swap_btc_to_usdt,  swap_btc_to_eth,    swap_btc_to_alpha,  swap_btc_to_near,
                swap_eth_to_usdt,  swap_eth_to_btc,    swap_eth_to_alpha,  swap_eth_to_near,
                swap_near_to_usdt, swap_near_to_alpha, swap_near_to_eth,   swap_near_to_btc
            )
        }
        _ => json!({"ok": false, "error": format!("unsupported export: {}", export)}),
    }
}
