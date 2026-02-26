mod assets;
mod commands;
mod geometry;
mod hud;
mod io;
mod loop_utils;
mod scenarios;
mod session;

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    Demo001,
    Demo002,
    Demo003,
    Demo004,
    Demo005,
    Demo006,
    Demo007,
    Demo008,
    Demo009,
    Demo010,
    Demo011,
    Demo012,
    Demo013,
    Demo014,
    Demo015,
    Demo016,
    Demo017,
    Demo018,
    Demo019,
    Demo020,
    Demo021,
    Demo022,
    Demo023,
    Demo024,
    Demo025,
    Demo026,
    Demo027,
    Demo028,
}

impl DemoKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "demo_001" | "demo001" | "1" => Some(Self::Demo001),
            "demo_002" | "demo002" | "2" => Some(Self::Demo002),
            "demo_003" | "demo003" | "3" => Some(Self::Demo003),
            "demo_004" | "demo004" | "4" => Some(Self::Demo004),
            "demo_005" | "demo005" | "5" => Some(Self::Demo005),
            "demo_006" | "demo006" | "6" => Some(Self::Demo006),
            "demo_007" | "demo007" | "7" => Some(Self::Demo007),
            "demo_008" | "demo008" | "8" => Some(Self::Demo008),
            "demo_009" | "demo009" | "9" => Some(Self::Demo009),
            "demo_010" | "demo010" | "10" => Some(Self::Demo010),
            "demo_011" | "demo011" | "11" => Some(Self::Demo011),
            "demo_012" | "demo012" | "12" => Some(Self::Demo012),
            "demo_013" | "demo013" | "13" => Some(Self::Demo013),
            "demo_014" | "demo014" | "14" => Some(Self::Demo014),
            "demo_015" | "demo015" | "15" => Some(Self::Demo015),
            "demo_016" | "demo016" | "16" => Some(Self::Demo016),
            "demo_017" | "demo017" | "17" => Some(Self::Demo017),
            "demo_018" | "demo018" | "18" => Some(Self::Demo018),
            "demo_019" | "demo019" | "19" => Some(Self::Demo019),
            "demo_020" | "demo020" | "20" => Some(Self::Demo020),
            "demo_021" | "demo021" | "21" => Some(Self::Demo021),
            "demo_022" | "demo022" | "22" => Some(Self::Demo022),
            "demo_023" | "demo023" | "23" => Some(Self::Demo023),
            "demo_024" | "demo024" | "24" => Some(Self::Demo024),
            "demo_025" | "demo025" | "25" => Some(Self::Demo025),
            "demo_026" | "demo026" | "26" => Some(Self::Demo026),
            "demo_027" | "demo027" | "27" => Some(Self::Demo027),
            "demo_028" | "demo028" | "28" => Some(Self::Demo028),
            _ => None,
        }
    }

    pub fn number(self) -> u32 {
        match self {
            Self::Demo001 => 1,
            Self::Demo002 => 2,
            Self::Demo003 => 3,
            Self::Demo004 => 4,
            Self::Demo005 => 5,
            Self::Demo006 => 6,
            Self::Demo007 => 7,
            Self::Demo008 => 8,
            Self::Demo009 => 9,
            Self::Demo010 => 10,
            Self::Demo011 => 11,
            Self::Demo012 => 12,
            Self::Demo013 => 13,
            Self::Demo014 => 14,
            Self::Demo015 => 15,
            Self::Demo016 => 16,
            Self::Demo017 => 17,
            Self::Demo018 => 18,
            Self::Demo019 => 19,
            Self::Demo020 => 20,
            Self::Demo021 => 21,
            Self::Demo022 => 22,
            Self::Demo023 => 23,
            Self::Demo024 => 24,
            Self::Demo025 => 25,
            Self::Demo026 => 26,
            Self::Demo027 => 27,
            Self::Demo028 => 28,
        }
    }

    pub fn title(self) -> String {
        format!("Vulfram Demo {:03}", self.number())
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
        println!("Selected demo from args: {:?}", demo);
        return demo;
    }

    if let Ok(value) = env::var("VULFRAM_DEMO")
        && let Some(demo) = DemoKind::from_str(&value)
    {
        println!("Selected demo from env: {:?}", demo);
        return demo;
    }

    DemoKind::Demo001
}

pub fn run_demo(demo: DemoKind, ctx: DemoContext) -> bool {
    scenarios::run(demo, ctx)
}

pub use io::send_commands;
pub use session::create_window;
