pub mod audio;
pub mod buffers;
pub mod cmd;
pub mod id_policy;
pub mod image;
pub mod input;
mod lifecycle;
pub mod platform;
pub mod platforms;
pub mod profiling;
mod queue;
pub mod realm;
pub mod render;
pub mod resources;
mod singleton;
mod state;
pub mod system;
pub mod target;
#[cfg(test)]
pub mod test_support;
mod tick;
pub mod time;
pub mod window;

// Re-exports for public API
#[allow(unused)]
pub use buffers::galfus_upload_buffer;
#[allow(unused)]
pub use lifecycle::{galfus_dispose, galfus_init};
#[allow(unused)]
pub use profiling::galfus_get_profiling;
#[allow(unused)]
pub use queue::{galfus_receive_events, galfus_receive_queue, galfus_send_queue};
#[allow(unused)]
pub use tick::galfus_tick;

#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(unused)]
pub enum GalfusResult {
    Success = 0,
    UnknownError,
    NotInitialized,
    AlreadyInitialized,
    WrongThread,
    NotInBrowser,
    CmdInvalidMessagePackError,
    BufferNotFound,
    BufferIdCollision,
    InvalidUploadType,
}
