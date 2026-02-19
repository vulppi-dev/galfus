use serde::{Deserialize, Serialize};

use crate::core::buffers::state::UploadType;
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use crate::core::ui::image_async::{UiImageAsyncEvent, UiImageDecodeJob};
use crate::core::ui::state::UiImageRecord;
use crate::core::ui::types::UiImageId;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiImageCreateFromBufferArgs {
    pub image_id: UiImageId,
    pub buffer_id: u64,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiImageCreateFromBuffer {
    pub success: bool,
    pub message: String,
    pub pending: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiImageDisposeArgs {
    pub image_id: UiImageId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiImageDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_ui_image_create_from_buffer(
    engine: &mut EngineState,
    args: &CmdUiImageCreateFromBufferArgs,
) -> CmdResultUiImageCreateFromBuffer {
    let ui_state = &mut engine.universal_state.ui;
    if ui_state.images.contains_key(&args.image_id)
        || ui_state.image_async.is_pending(args.image_id)
    {
        return CmdResultUiImageCreateFromBuffer {
            success: false,
            message: format!("UiImage {} already exists or pending", args.image_id),
            pending: false,
        };
    }

    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(buffer) => buffer,
        None => {
            return CmdResultUiImageCreateFromBuffer {
                success: false,
                message: format!("Buffer {} not found", args.buffer_id),
                pending: false,
            };
        }
    };

    if buffer.upload_type != UploadType::ImageData {
        return CmdResultUiImageCreateFromBuffer {
            success: false,
            message: format!(
                "Invalid buffer type. Expected ImageData, got {:?}",
                buffer.upload_type
            ),
            pending: false,
        };
    }

    let job = UiImageDecodeJob {
        image_id: args.image_id,
        label: args.label.clone(),
        bytes: buffer.data,
    };

    if let Err(message) = ui_state.image_async.enqueue(job) {
        return CmdResultUiImageCreateFromBuffer {
            success: false,
            message,
            pending: false,
        };
    }

    CmdResultUiImageCreateFromBuffer {
        success: true,
        message: "UI image decode queued".into(),
        pending: true,
    }
}

pub fn engine_cmd_ui_image_dispose(
    engine: &mut EngineState,
    args: &CmdUiImageDisposeArgs,
) -> CmdResultUiImageDispose {
    let ui_state = &mut engine.universal_state.ui;
    ui_state.image_async.cancel(args.image_id);
    if ui_state.images.remove(&args.image_id).is_none() {
        return CmdResultUiImageDispose {
            success: false,
            message: format!("UiImage {} not found", args.image_id),
        };
    }
    CmdResultUiImageDispose {
        success: true,
        message: "UI image disposed".into(),
    }
}

pub fn process_async_ui_image_results(engine: &mut EngineState) {
    let ui_state = &mut engine.universal_state.ui;
    let results = ui_state.image_async.drain_results();
    for result in results {
        match result {
            UiImageAsyncEvent::Started {
                image_id,
                total_bytes,
            } => {
                engine
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        SystemEvent::UiImageProcessingStarted {
                            image_id,
                            total_bytes,
                        },
                    ));
            }
            UiImageAsyncEvent::Progress {
                image_id,
                processed_bytes,
                total_bytes,
            } => {
                engine
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        SystemEvent::UiImageProcessingProgress {
                            image_id,
                            processed_bytes,
                            total_bytes,
                        },
                    ));
            }
            UiImageAsyncEvent::Finished {
                image_id,
                success,
                message,
                total_bytes,
            } => {
                engine
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        SystemEvent::UiImageProcessingFinished {
                            image_id,
                            success,
                            message,
                            total_bytes,
                        },
                    ));
            }
            UiImageAsyncEvent::Result(result) => {
                if ui_state.image_async.was_canceled(result.image_id) {
                    engine
                        .event_queue
                        .push(crate::core::cmd::EngineEvent::System(
                            SystemEvent::UiImageReady {
                                image_id: result.image_id,
                                success: false,
                                message: "UI image decode canceled".into(),
                            },
                        ));
                    continue;
                }

                if let Some(image) = result.image {
                    let size = [image.width, image.height];
                    ui_state.images.insert(
                        result.image_id,
                        UiImageRecord {
                            label: result.label,
                            image,
                            size,
                            texture: None,
                        },
                    );
                    engine
                        .event_queue
                        .push(crate::core::cmd::EngineEvent::System(
                            SystemEvent::UiImageReady {
                                image_id: result.image_id,
                                success: true,
                                message: "UI image decoded".into(),
                            },
                        ));
                } else {
                    engine
                        .event_queue
                        .push(crate::core::cmd::EngineEvent::System(
                            SystemEvent::UiImageReady {
                                image_id: result.image_id,
                                success: false,
                                message: result.message,
                            },
                        ));
                }
            }
        }
    }
}
