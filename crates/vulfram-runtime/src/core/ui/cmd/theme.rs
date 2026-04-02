use crate::core::state::EngineState;
use crate::core::ui::state::UiThemeState;
pub use vulfram_realm_ui::{
    CmdResultUiThemeDefine, CmdResultUiThemeDispose, CmdUiThemeDefineArgs, CmdUiThemeDisposeArgs,
};

pub fn engine_cmd_ui_theme_define(
    engine: &mut EngineState,
    args: &CmdUiThemeDefineArgs,
) -> CmdResultUiThemeDefine {
    let ui_state = &mut engine.universal_state.interaction.ui;
    let version = match (args.version, ui_state.themes.get(&args.theme_id)) {
        (Some(version), _) => version,
        (None, Some(existing)) => existing.version.saturating_add(1),
        (None, None) => 1,
    };
    ui_state.themes.insert(
        args.theme_id,
        UiThemeState {
            version,
            data: args.data.clone(),
            font_data: args.font_data.clone(),
            font_families: args.font_families.clone(),
        },
    );
    CmdResultUiThemeDefine {
        success: true,
        message: "UI theme defined".into(),
        theme_id: Some(args.theme_id),
        version: Some(version),
    }
}

pub fn engine_cmd_ui_theme_dispose(
    engine: &mut EngineState,
    args: &CmdUiThemeDisposeArgs,
) -> CmdResultUiThemeDispose {
    let ui_state = &mut engine.universal_state.interaction.ui;
    if ui_state.themes.remove(&args.theme_id).is_none() {
        return CmdResultUiThemeDispose {
            success: false,
            message: format!("UiTheme {} not found", args.theme_id),
        };
    }
    CmdResultUiThemeDispose {
        success: true,
        message: "UI theme disposed".into(),
    }
}
