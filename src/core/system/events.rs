use serde::{Deserialize, Serialize};

/// System-level events
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum SystemEvent {
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
    UiImageProcessingStarted {
        image_id: u32,
        total_bytes: u64,
    },
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
}
