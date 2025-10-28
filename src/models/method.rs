use fluent_uri::Uri;
use serde::{Deserialize, Serialize};
use crate::models::agent::{AgentId, AppId, ExecKind, Port, Visibility};

pub type CodeUri = Uri<String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgramAbi {
    ComponentBytesIo,
    WasiJsonStdio,
    HttpJson,
    LocalFn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramRef {
    pub module_uri: CodeUri,   // where to fetch/execute the program (file://, http://, etc.)
    pub export: String,        // symbol or route name, e.g., "add", "to_upper", "main"
    pub abi: ProgramAbi,       // LocalFn | HttpJson | WasiJsonStdio | ComponentBytesIo
    pub checksum: String,      // integrity hash of the program bytes (e.g., blake3/sha256 hex)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Method {
    pub id: AgentId,
    pub app_id: AppId,
    pub label: String,
    pub version: String,
    pub visibility: Visibility,
    pub exec_kind: ExecKind,
    pub code_uri: CodeUri,       // required (no Option) for minimal demo
    pub description: String,
    pub in_port: Port,
    pub out_port: Port,
    pub program: ProgramRef,     // binding for executor (LocalFn/HttpJson/WasiJsonStdio)
    pub embedding: Vec<f32>,
}