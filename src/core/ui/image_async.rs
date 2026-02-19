use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::core::image::{ImageBuffer, ImageDecoder};
use crate::core::ui::types::UiImageId;

#[derive(Debug, Clone)]
pub struct UiImageDecodeJob {
    pub image_id: UiImageId,
    pub label: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct UiImageDecodeResult {
    pub image_id: UiImageId,
    pub label: Option<String>,
    pub image: Option<ImageBuffer>,
    pub message: String,
}

#[derive(Debug)]
pub enum UiImageAsyncEvent {
    Started {
        image_id: UiImageId,
        total_bytes: u64,
    },
    Progress {
        image_id: UiImageId,
        processed_bytes: u64,
        total_bytes: u64,
    },
    Finished {
        image_id: UiImageId,
        success: bool,
        message: String,
        total_bytes: u64,
    },
    Result(UiImageDecodeResult),
}

#[derive(Debug)]
pub struct UiImageAsyncManager {
    sender: Sender<UiImageAsyncEvent>,
    receiver: Receiver<UiImageAsyncEvent>,
    pending: HashSet<UiImageId>,
    canceled: HashSet<UiImageId>,
}

impl UiImageAsyncManager {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver,
            pending: HashSet::new(),
            canceled: HashSet::new(),
        }
    }

    pub fn is_pending(&self, image_id: UiImageId) -> bool {
        self.pending.contains(&image_id)
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn pending_ids(&self) -> HashSet<UiImageId> {
        self.pending.iter().copied().collect()
    }

    pub fn enqueue(&mut self, job: UiImageDecodeJob) -> Result<(), String> {
        if !self.pending.insert(job.image_id) {
            return Err(format!("UiImage {} is already pending", job.image_id));
        }
        self.canceled.remove(&job.image_id);
        let sender = self.sender.clone();
        spawn_decode(job, sender);
        Ok(())
    }

    pub fn cancel(&mut self, image_id: UiImageId) {
        self.pending.remove(&image_id);
        self.canceled.insert(image_id);
    }

    pub fn was_canceled(&mut self, image_id: UiImageId) -> bool {
        self.canceled.remove(&image_id)
    }

    pub fn drain_results(&mut self) -> Vec<UiImageAsyncEvent> {
        let mut results = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            if let UiImageAsyncEvent::Result(decoded) = &result {
                self.pending.remove(&decoded.image_id);
            }
            results.push(result);
        }
        results
    }
}

impl Default for UiImageAsyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "wasm"))]
fn spawn_decode(job: UiImageDecodeJob, sender: Sender<UiImageAsyncEvent>) {
    std::thread::spawn(move || {
        let total_bytes = job.bytes.len() as u64;
        let _ = sender.send(UiImageAsyncEvent::Started {
            image_id: job.image_id,
            total_bytes,
        });
        let _ = sender.send(UiImageAsyncEvent::Progress {
            image_id: job.image_id,
            processed_bytes: 0,
            total_bytes,
        });
        let image = ImageDecoder::try_decode(&job.bytes);
        let message = if image.is_some() {
            "UI image decoded".to_string()
        } else {
            "Failed to decode UI image".to_string()
        };
        let _ = sender.send(UiImageAsyncEvent::Progress {
            image_id: job.image_id,
            processed_bytes: total_bytes,
            total_bytes,
        });
        let _ = sender.send(UiImageAsyncEvent::Finished {
            image_id: job.image_id,
            success: image.is_some(),
            message: message.clone(),
            total_bytes,
        });
        let _ = sender.send(UiImageAsyncEvent::Result(UiImageDecodeResult {
            image_id: job.image_id,
            label: job.label,
            image,
            message,
        }));
    });
}

#[cfg(feature = "wasm")]
fn spawn_decode(job: UiImageDecodeJob, sender: Sender<UiImageAsyncEvent>) {
    wasm_bindgen_futures::spawn_local(async move {
        let total_bytes = job.bytes.len() as u64;
        let _ = sender.send(UiImageAsyncEvent::Started {
            image_id: job.image_id,
            total_bytes,
        });
        let _ = sender.send(UiImageAsyncEvent::Progress {
            image_id: job.image_id,
            processed_bytes: 0,
            total_bytes,
        });
        let image = ImageDecoder::try_decode(&job.bytes);
        let message = if image.is_some() {
            "UI image decoded".to_string()
        } else {
            "Failed to decode UI image".to_string()
        };
        let _ = sender.send(UiImageAsyncEvent::Progress {
            image_id: job.image_id,
            processed_bytes: total_bytes,
            total_bytes,
        });
        let _ = sender.send(UiImageAsyncEvent::Finished {
            image_id: job.image_id,
            success: image.is_some(),
            message: message.clone(),
            total_bytes,
        });
        let _ = sender.send(UiImageAsyncEvent::Result(UiImageDecodeResult {
            image_id: job.image_id,
            label: job.label,
            image,
            message,
        }));
    });
}
