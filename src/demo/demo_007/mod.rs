mod maps;
mod runtime;
mod setup;
mod ui;

use crate::demo::DemoContext;

pub fn run(ctx: DemoContext) -> bool {
    let setup = setup::Demo007Setup::new();
    let _realms = setup.apply(ctx);
    runtime::run(ctx, &setup)
}
