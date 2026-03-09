use serde::{Deserialize, Serialize};

use crate::core::target::TargetId;

fn default_true() -> bool {
    true
}

fn default_scope() -> TargetListenerScope {
    TargetListenerScope::Target
}

const fn u8_100() -> u8 {
    100
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TargetListenerScope {
    Target,
    TargetAndDescendants,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerUpsertArgs {
    pub listener_id: u64,
    pub target_id: u64,
    #[serde(default)]
    pub window_id: Option<u32>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_scope")]
    pub scope: TargetListenerScope,
    #[serde(default)]
    pub throttle_ms: u32,
    #[serde(default = "u8_100")]
    pub sample_percent: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerDisposeArgs {
    pub listener_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerListArgs {
    #[serde(default)]
    pub target_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTargetListenerSnapshot {
    pub listener_id: u64,
    pub target_id: u64,
    pub window_id: Option<u32>,
    pub enabled: bool,
    pub events: Vec<String>,
    pub scope: TargetListenerScope,
    pub throttle_ms: u32,
    pub sample_percent: u8,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultInputTargetListenerList {
    pub success: bool,
    pub message: String,
    pub listeners: Vec<InputTargetListenerSnapshot>,
}

#[derive(Debug, Clone)]
pub struct InputTargetListenerConfig {
    pub listener_id: u64,
    pub target_id: TargetId,
    pub window_id: Option<u32>,
    pub enabled: bool,
    pub events: Vec<String>,
    pub scope: TargetListenerScope,
    pub throttle_ms: u32,
    pub sample_percent: u8,
}
