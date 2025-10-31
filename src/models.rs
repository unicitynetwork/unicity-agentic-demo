use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

mod agent;
mod method;
mod port;

pub use agent::{Agent, CreateAgent};
pub use method::{CreateMethod, ExecKind, Method, ProgramAbi, ProgramRef, Visibility};
pub use port::{Channel, CreatePort, Port};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
}
