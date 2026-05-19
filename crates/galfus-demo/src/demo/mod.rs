mod io;
mod scenarios;
mod session;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    FrameGraph001,
}

impl DemoKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "1" | "001" | "demo001" | "demo_001" | "framegraph" | "framegraph001" => {
                Some(Self::FrameGraph001)
            }
            _ => None,
        }
    }

    pub fn title(self) -> String {
        match self {
            Self::FrameGraph001 => "Galfus Demo 001 - FrameGraph".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DemoContext {
    pub window_id: u32,
    pub realm_id: u32,
}

pub fn select_demo() -> DemoKind {
    if let Some(arg) = env::args().nth(1)
        && let Some(demo) = DemoKind::from_str(&arg)
    {
        return demo;
    }

    if let Ok(value) = env::var("GALFUS_DEMO")
        && let Some(demo) = DemoKind::from_str(&value)
    {
        return demo;
    }

    DemoKind::FrameGraph001
}

pub fn run_demo(demo: DemoKind, ctx: DemoContext) -> bool {
    scenarios::run(demo, ctx)
}

pub use io::send_commands;
pub use session::create_window;
