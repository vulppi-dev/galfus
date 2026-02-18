use crate::core::cmd::EngineCmd;
use crate::core::input::events::PointerTraceLevel;
use crate::core::ui::cmd::CmdUiEventTraceSetArgs;
use crate::demo::DemoContext;
use crate::demo::io::send_commands;

pub fn run(ctx: DemoContext) -> bool {
    let _ = send_commands(vec![EngineCmd::CmdUiEventTraceSet(
        CmdUiEventTraceSetArgs {
            level: Some(PointerTraceLevel::Full),
            sampling_percent: Some(100),
        },
    )]);
    super::demo_006::run(ctx)
}
