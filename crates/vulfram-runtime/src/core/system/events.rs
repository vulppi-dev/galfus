use serde::{Deserialize, Serialize};

use crate::core::input::events::ElementState;
use vulfram_protocol::{UiViewportClass, UiViewportCommand};

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

    /// Async UI image decode finished
    UiImageReady {
        image_id: u32,
        success: bool,
        message: String,
    },
    /// Async UI image decode started
    UiImageProcessingStarted { image_id: u32, total_bytes: u64 },
    /// Async UI image decode progress
    UiImageProcessingProgress {
        image_id: u32,
        processed_bytes: u64,
        total_bytes: u64,
    },
    /// Async UI image decode finished
    UiImageProcessingFinished {
        image_id: u32,
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

    /// UI requested opening a URL; host decides policy and execution.
    #[serde(rename_all = "camelCase")]
    UiOpenUrl {
        window_id: u32,
        realm_id: u32,
        url: String,
        new_tab: bool,
    },

    /// UI requested writing text to host clipboard.
    #[serde(rename_all = "camelCase")]
    UiClipboardSetText {
        window_id: u32,
        realm_id: u32,
        text: String,
    },

    /// UI requested host copy command.
    #[serde(rename_all = "camelCase")]
    UiClipboardRequestCopy { window_id: u32, realm_id: u32 },

    /// UI requested host cut command.
    #[serde(rename_all = "camelCase")]
    UiClipboardRequestCut { window_id: u32, realm_id: u32 },

    /// UI requested host paste command.
    #[serde(rename_all = "camelCase")]
    UiClipboardRequestPaste { window_id: u32, realm_id: u32 },

    /// UI requested screenshot capture for the current viewport.
    #[serde(rename_all = "camelCase")]
    UiScreenshotRequest { window_id: u32, realm_id: u32 },

    /// UI viewport sync event kept for host compatibility during vNext transition.
    #[serde(rename_all = "camelCase")]
    UiViewportSync {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        parent_viewport_id: Option<u64>,
        class: UiViewportClass,
        title: Option<String>,
    },

    /// UI viewport command not handled natively by this runtime (or for non-root viewport).
    #[serde(rename_all = "camelCase")]
    UiViewportCommand {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        command: UiViewportCommand,
    },

    /// Runtime fallback mode for additional viewports when native multi-viewport is unavailable.
    #[serde(rename_all = "camelCase")]
    UiViewportFallbackEmbedded {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        parent_viewport_id: Option<u64>,
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
}
