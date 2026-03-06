use serde::{Deserialize, Serialize};

mod close;
mod create;
mod cursor;
mod measurement;
mod state;

pub use close::*;
pub use create::*;
pub use cursor::*;
pub use measurement::*;
pub use state::*;

// Shared types
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EngineWindowState {
    Minimized = 0,
    Maximized,
    Windowed,
    Fullscreen,
    WindowedFullscreen,
}

impl Default for EngineWindowState {
    fn default() -> Self {
        EngineWindowState::Windowed
    }
}

fn window_size_default() -> glam::UVec2 {
    glam::UVec2::new(800, 600)
}
