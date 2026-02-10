use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiEventKind {
    Click,
    ChangeCommit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UiEvent {
    pub realm_id: u32,
    pub document_id: u32,
    pub node_id: u32,
    pub kind: UiEventKind,
    #[serde(default)]
    pub label: Option<String>,
}
