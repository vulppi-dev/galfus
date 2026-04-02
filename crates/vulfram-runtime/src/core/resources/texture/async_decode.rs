use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::core::image::{ImageBuffer, ImageDecoder};
use crate::core::resources::texture::{ForwardAtlasOptions, TextureCreateMode};

#[derive(Debug, Clone)]
pub struct TextureDecodeJob {
    pub texture_id: u32,
    pub label: Option<String>,
    pub srgb: Option<bool>,
    pub mode: TextureCreateMode,
    pub atlas_options: Option<ForwardAtlasOptions>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct TextureDecodeResult {
    pub texture_id: u32,
    pub label: Option<String>,
    pub srgb: Option<bool>,
    pub mode: TextureCreateMode,
    pub atlas_options: Option<ForwardAtlasOptions>,
    pub image: Option<ImageBuffer>,
    pub message: String,
}

#[derive(Debug)]
pub enum TextureAsyncEvent {
    Started {
        texture_id: u32,
        total_bytes: u64,
    },
    Progress {
        texture_id: u32,
        processed_bytes: u64,
        total_bytes: u64,
    },
    Finished {
        texture_id: u32,
        success: bool,
        message: String,
        total_bytes: u64,
    },
    Result(TextureDecodeResult),
}

pub struct TextureAsyncManager {
    sender: Sender<TextureAsyncEvent>,
    receiver: Receiver<TextureAsyncEvent>,
    pending: HashSet<u32>,
    canceled: HashSet<u32>,
}

impl TextureAsyncManager {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver,
            pending: HashSet::new(),
            canceled: HashSet::new(),
        }
    }

    pub fn is_pending(&self, texture_id: u32) -> bool {
        self.pending.contains(&texture_id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn enqueue(&mut self, job: TextureDecodeJob) -> Result<(), String> {
        if !self.pending.insert(job.texture_id) {
            return Err(format!("Texture {} is already pending", job.texture_id));
        }
        self.canceled.remove(&job.texture_id);
        let sender = self.sender.clone();
        spawn_decode(job, sender);
        Ok(())
    }

    pub fn cancel(&mut self, texture_id: u32) -> bool {
        let removed = self.pending.remove(&texture_id);
        if removed {
            self.canceled.insert(texture_id);
        }
        removed
    }

    pub fn was_canceled(&mut self, texture_id: u32) -> bool {
        self.canceled.remove(&texture_id)
    }

    pub fn drain_results(&mut self) -> Vec<TextureAsyncEvent> {
        let mut results = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            if let TextureAsyncEvent::Result(decoded) = &result {
                self.pending.remove(&decoded.texture_id);
            }
            results.push(result);
        }
        results
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_decode(job: TextureDecodeJob, sender: Sender<TextureAsyncEvent>) {
    std::thread::spawn(move || {
        let total_bytes = job.bytes.len() as u64;
        let _ = sender.send(TextureAsyncEvent::Started {
            texture_id: job.texture_id,
            total_bytes,
        });
        let _ = sender.send(TextureAsyncEvent::Progress {
            texture_id: job.texture_id,
            processed_bytes: 0,
            total_bytes,
        });
        let image = ImageDecoder::try_decode(&job.bytes);
        let message = if image.is_some() {
            "Texture decoded".to_string()
        } else {
            "Failed to decode image".to_string()
        };
        let _ = sender.send(TextureAsyncEvent::Progress {
            texture_id: job.texture_id,
            processed_bytes: total_bytes,
            total_bytes,
        });
        let _ = sender.send(TextureAsyncEvent::Finished {
            texture_id: job.texture_id,
            success: image.is_some(),
            message: message.clone(),
            total_bytes,
        });
        let _ = sender.send(TextureAsyncEvent::Result(TextureDecodeResult {
            texture_id: job.texture_id,
            label: job.label,
            srgb: job.srgb,
            mode: job.mode,
            atlas_options: job.atlas_options,
            image,
            message,
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_only_marks_existing_pending_entries() {
        let mut manager = TextureAsyncManager::new();
        assert!(!manager.cancel(10));
        assert!(!manager.was_canceled(10));

        manager.pending.insert(10);
        assert!(manager.cancel(10));
        assert!(manager.was_canceled(10));
        assert!(!manager.was_canceled(10));
    }
}

#[cfg(target_arch = "wasm32")]
fn spawn_decode(job: TextureDecodeJob, sender: Sender<TextureAsyncEvent>) {
    wasm_bindgen_futures::spawn_local(async move {
        let total_bytes = job.bytes.len() as u64;
        let _ = sender.send(TextureAsyncEvent::Started {
            texture_id: job.texture_id,
            total_bytes,
        });
        let _ = sender.send(TextureAsyncEvent::Progress {
            texture_id: job.texture_id,
            processed_bytes: 0,
            total_bytes,
        });
        let image = ImageDecoder::try_decode(&job.bytes);
        let message = if image.is_some() {
            "Texture decoded".to_string()
        } else {
            "Failed to decode image".to_string()
        };
        let _ = sender.send(TextureAsyncEvent::Progress {
            texture_id: job.texture_id,
            processed_bytes: total_bytes,
            total_bytes,
        });
        let _ = sender.send(TextureAsyncEvent::Finished {
            texture_id: job.texture_id,
            success: image.is_some(),
            message: message.clone(),
            total_bytes,
        });
        let _ = sender.send(TextureAsyncEvent::Result(TextureDecodeResult {
            texture_id: job.texture_id,
            label: job.label,
            srgb: job.srgb,
            mode: job.mode,
            atlas_options: job.atlas_options,
            image,
            message,
        }));
    });
}
