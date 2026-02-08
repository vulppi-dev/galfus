mod config;
mod graph;
mod runtime;
mod setup;

use crate::demo::DemoContext;

pub fn run(ctx: DemoContext) -> bool {
    let setup = setup::Demo004Setup::new();
    setup.apply(ctx);
    runtime::run(ctx, &setup)
}
