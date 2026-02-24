mod runtime;
mod setup;

use crate::demo::DemoContext;

/// Demo 005 is the baseline multi-window setup:
/// - 2 windows
/// - 2 host 3D realms (one per window)
/// - no stress target/realm topology
pub fn run(ctx: DemoContext) -> bool {
    let setup = setup::Demo005Setup::new();
    let realms = setup.apply(ctx);
    runtime::run(ctx, &setup, &realms)
}
