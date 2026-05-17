use crate::demo::{DemoContext, DemoKind};

pub fn run(_demo: DemoKind, _ctx: DemoContext) -> bool {
    let _ = _ctx.window_id;
    let _ = _ctx.realm_id;
    false
}
