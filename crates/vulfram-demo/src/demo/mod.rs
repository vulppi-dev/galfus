mod io;
mod scenarios;
mod session;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    Demo1,
    Demo2,
    Demo3,
    Demo4,
    Demo5,
    Demo6,
    Demo7,
}

impl DemoKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "suite_a" | "suitea" | "a" | "1" | "demo_001" | "demo001" | "demo_002" | "demo002"
            | "demo_003" | "demo003" | "demo_004" | "demo004" | "demo_005" | "demo005" => {
                Some(Self::Demo1)
            }
            "suite_b" | "suiteb" | "b" | "2" | "demo_006" | "demo006" | "demo_007" | "demo007"
            | "demo_008" | "demo008" | "demo_009" | "demo009" => Some(Self::Demo2),
            "suite_c" | "suitec" | "c" | "3" | "demo_010" | "demo010" | "demo_011" | "demo011"
            | "demo_012" | "demo012" | "demo_013" | "demo013" | "demo_014" | "demo014"
            | "demo_015" | "demo015" | "demo_016" | "demo016" => Some(Self::Demo3),
            "suite_d" | "suited" | "d" | "4" | "demo_017" | "demo017" | "demo_018" | "demo018"
            | "demo_019" | "demo019" | "demo_020" | "demo020" | "demo_021" | "demo021" => {
                Some(Self::Demo4)
            }
            "suite_e" | "suitee" | "e" | "5" | "demo_022" | "demo022" | "demo_023" | "demo023"
            | "demo_024" | "demo024" | "demo_025" | "demo025" => Some(Self::Demo5),
            "suite_f" | "suitef" | "f" | "6" | "demo_026" | "demo026" | "demo_027" | "demo027"
            | "demo_028" | "demo028" => Some(Self::Demo6),
            "suite_g" | "suiteg" | "g" | "7" | "demo_029" | "demo029" => Some(Self::Demo7),
            _ => None,
        }
    }

    pub fn number(self) -> u32 {
        match self {
            Self::Demo1 => 1,
            Self::Demo2 => 2,
            Self::Demo3 => 3,
            Self::Demo4 => 4,
            Self::Demo5 => 5,
            Self::Demo6 => 6,
            Self::Demo7 => 7,
        }
    }

    pub fn title(self) -> String {
        format!("Vulfram Demo {}", self.number())
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

    if let Ok(value) = env::var("VULFRAM_DEMO")
        && let Some(demo) = DemoKind::from_str(&value)
    {
        return demo;
    }

    DemoKind::Demo1
}

pub fn run_demo(demo: DemoKind, ctx: DemoContext) -> bool {
    scenarios::run(demo, ctx)
}

pub use io::send_commands;
pub use session::create_window;
