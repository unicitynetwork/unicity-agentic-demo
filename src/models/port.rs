use fluent_uri::Uri;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

pub type TypeUri = Uri<String>;
pub type EndPointUri = Uri<String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Channel {
    Call,
    View,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreatePort {
    pub label: String,
    pub description: String,
    pub channel: Channel,
    pub type_uri: Option<TypeUri>,
    pub end_point: EndPointUri,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Port {
    pub r#in: RecordId,
    pub out: RecordId,
    pub label: String,
    pub description: String,
    pub channel: Channel,
    pub type_uri: Option<TypeUri>,
    pub end_point: EndPointUri,
}
