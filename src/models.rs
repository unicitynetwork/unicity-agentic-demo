use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

mod agent;
mod method;
mod port;

pub use agent::{Agent, CreateAgent};
pub use method::{CreateMethod, Visibility, ExecKind, ProgramRef, ProgramAbi, Method};
pub use port::{CreatePort, Port, Channel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
}
