use serde::{Deserialize, Serialize};

use crate::core::input::events::ElementState;

/// System-level events
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum SystemEvent {
    /// Diagnostic error forwarded to host event pool
    Error {
        scope: String,
        message: String,
        command_id: Option<u64>,
        command_type: Option<String>,
    },

    /// Command accepted but waiting for dependencies (eventual consistency queue).
    #[serde(rename_all = "camelCase")]
    CommandDeferred {
        command_id: u64,
        command_type: String,
        attempts: u32,
        reason: String,
    },

    /// Deferred command was eventually applied.
    #[serde(rename_all = "camelCase")]
    CommandApplied {
        command_id: u64,
        command_type: String,
        attempts: u32,
    },

    /// Deferred command expired and was dropped from retry queue.
    #[serde(rename_all = "camelCase")]
    CommandDropped {
        command_id: u64,
        command_type: String,
        attempts: u32,
        reason: String,
    },

    /// Application was resumed (from suspended state)
    OnResume,

    /// Application was suspended
    OnSuspend,

    /// Low memory warning
    OnMemoryWarning,

    /// Application is about to exit
    OnExit,

    /// Notification was clicked
    OnNotificationClicked { id: String },

    /// Notification was dismissed or expired
    OnNotificationDismissed { id: String },

    /// Async texture decode finished
    TextureReady {
        window_id: u32,
        texture_id: u32,
        success: bool,
        message: String,
    },
    /// Async texture decode started
    TextureProcessingStarted {
        window_id: u32,
        texture_id: u32,
        total_bytes: u64,
    },
    /// Async texture decode progress
    TextureProcessingProgress {
        window_id: u32,
        texture_id: u32,
        processed_bytes: u64,
        total_bytes: u64,
    },
    /// Async texture decode finished
    TextureProcessingFinished {
        window_id: u32,
        texture_id: u32,
        success: bool,
        message: String,
        total_bytes: u64,
    },

    /// Async audio decode finished
    AudioReady {
        resource_id: u32,
        success: bool,
        message: String,
    },

    /// Async audio stream progress
    AudioStreamProgress {
        resource_id: u32,
        received_bytes: u64,
        total_bytes: u64,
        complete: bool,
    },

    /// Input event matched by an input-target-listener.
    #[serde(rename_all = "camelCase")]
    InputTargetListenerEvent {
        listener_id: u64,
        target_id: u64,
        event_type: String,
        window_id: Option<u32>,
        window_width: Option<u32>,
        window_height: Option<u32>,
        pointer_id: Option<u64>,
        position_global: Option<glam::Vec2>,
        position_target: Option<glam::Vec2>,
        target_width: Option<u32>,
        target_height: Option<u32>,
        key_code: Option<u32>,
        key_state: Option<ElementState>,
    },

    #[serde(rename_all = "camelCase")]
    ResourceMutation {
        kind: String,
        id: u64,
        action: String,
        realm_id: Option<u32>,
        window_id: Option<u32>,
        revision: u64,
    },

    #[serde(rename_all = "camelCase")]
    MaterialInstanceFallbackApplied {
        material_id: u32,
        previous_definition_id: u32,
        fallback_definition_id: u32,
        reason: String,
    },
}
