use fluent_uri::Uri;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use crate::models::agent::AgentId;
use crate::models::CreatePort;

pub type MethodId = RecordId;
pub type CodeUri = Uri<String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

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
pub enum ExecKind {
    Local,
    Http,
    Wasm,
    Noop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMethod {
    pub agent: AgentId,
    pub label: String,
    pub version: String,
    pub visibility: Visibility,
    pub exec_kind: ExecKind,
    pub description: String,
    pub in_port: CreatePort,
    pub out_port: CreatePort,
    pub program: ProgramRef,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Method {
    pub id: MethodId,
    pub agent: AgentId,
    pub label: String,
    pub version: String,
    pub visibility: Visibility,
    pub exec_kind: ExecKind,
    pub description: String,
    pub in_port: CreatePort,
    pub out_port: CreatePort,
    pub program: ProgramRef,
    pub embedding: Vec<f32>,
}