mod scenario_001_frame_graph;

use crate::demo::{DemoContext, DemoKind};

pub fn run(demo: DemoKind, ctx: DemoContext) -> bool {
    match demo {
        DemoKind::FrameGraph001 => scenario_001_frame_graph::run(ctx),
    }
}
