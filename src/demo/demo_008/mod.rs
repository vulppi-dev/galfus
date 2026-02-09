mod graph;
mod runtime;
mod setup;

use crate::demo::DemoContext;

pub fn run(ctx: DemoContext) -> bool {
    let setup = setup::Demo008Setup::new();
    let realms = setup.apply(ctx);
    runtime::run(ctx, &setup, &realms)
}
