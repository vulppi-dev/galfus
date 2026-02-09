mod runtime;
mod setup;
mod maps;

use crate::demo::DemoContext;

pub fn run(ctx: DemoContext) -> bool {
    let setup = setup::Demo008Setup::new();
    let realms = setup.apply(ctx);
    runtime::run(ctx, &setup, &realms)
}
