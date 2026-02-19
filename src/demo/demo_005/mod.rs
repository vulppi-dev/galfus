mod maps;
mod runtime;
mod setup;

use crate::demo::DemoContext;

pub fn run(ctx: DemoContext) -> bool {
    let setup = setup::Demo005Setup::new();
    let realms = setup.apply(ctx);
    runtime::run(ctx, &setup, &realms)
}
