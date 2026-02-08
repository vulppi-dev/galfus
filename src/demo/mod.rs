mod assets;
mod commands;
mod demo_001;
mod demo_002;
mod demo_003;
mod demo_004;
mod geometry;
mod io;
mod loop_utils;
mod session;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    Demo001,
    Demo002,
    Demo003,
    Demo004,
}

impl DemoKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "demo_001" | "demo001" | "1" => Some(Self::Demo001),
            "demo_002" | "demo002" | "2" => Some(Self::Demo002),
            "demo_003" | "demo003" | "3" => Some(Self::Demo003),
            "demo_004" | "demo004" | "4" => Some(Self::Demo004),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Demo001 => "Vulfram Demo 001",
            Self::Demo002 => "Vulfram Demo 002",
            Self::Demo003 => "Vulfram Demo 003",
            Self::Demo004 => "Vulfram Demo 004",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DemoContext {
    pub window_id: u32,
    pub realm_id: u32,
}

pub fn select_demo() -> DemoKind {
    if let Some(arg) = env::args().nth(1) {
        if let Some(demo) = DemoKind::from_str(&arg) {
            println!("Selected demo from args: {:?}", demo);
            return demo;
        }
    }

    if let Ok(value) = env::var("VULFRAM_DEMO") {
        if let Some(demo) = DemoKind::from_str(&value) {
            println!("Selected demo from env: {:?}", demo);
            return demo;
        }
    }

    DemoKind::Demo001
}

pub fn run_demo(demo: DemoKind, ctx: DemoContext) -> bool {
    match demo {
        DemoKind::Demo001 => demo_001::run(ctx),
        DemoKind::Demo002 => demo_002::run(ctx),
        DemoKind::Demo003 => demo_003::run(ctx),
        DemoKind::Demo004 => demo_004::run(ctx),
    }
}

pub use assets::{
    load_texture_bytes, upload_binary_bytes, upload_buffer, upload_texture, upload_texture_bytes,
};
pub use commands::{
    create_ambient_light_cmd, create_camera_cmd, create_floor_cmd, create_instanced_cubes,
    create_point_light_cmd, create_shadow_config_cmd, create_standard_material_cmd,
    create_texture_cmd, draw_axes_gizmos, default_camera_transform, CubeData,
};
pub use geometry::build_skinned_plane;
pub use io::{receive_events, receive_responses, send_commands};
pub use loop_utils::{run_loop, run_loop_with_events};
pub use session::create_window;
