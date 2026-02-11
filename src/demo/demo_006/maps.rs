use glam::Vec4;

use crate::core::cmd::EngineCmd;
use crate::core::target::cmd::{CmdTargetBindUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{TargetBindLayout, TargetKind};

#[derive(Debug, Clone, Copy)]
pub struct Demo006TargetIds {
    pub window_main: u64,
    pub panel_ui: u64,
    pub texture_ui: u64,
    pub texture_view: u64,
    pub texture_panel_3d: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo006BindRealms {
    pub host_main: u32,
    pub ui: u32,
    pub ui_panel_3d: u32,
    pub view: u32,
}

pub fn build_target_cmds(window_main: u32) -> (Demo006TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo006TargetIds {
        window_main: 9200,
        panel_ui: 9201,
        texture_ui: 9202,
        texture_view: 9203,
        texture_panel_3d: 9204,
    };

    let targets = vec![
        CmdTargetUpsertArgs {
            target_id: target_ids.window_main,
            kind: TargetKind::Window,
            owner_window_id: Some(window_main),
            size_override: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.panel_ui,
            kind: TargetKind::PanelEmbed,
            owner_window_id: Some(window_main),
            size_override: Some(glam::UVec2::new(520, 320)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.texture_ui,
            kind: TargetKind::Texture,
            owner_window_id: None,
            size_override: Some(glam::UVec2::new(520, 320)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.texture_view,
            kind: TargetKind::Texture,
            owner_window_id: None,
            size_override: Some(glam::UVec2::new(384, 256)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.texture_panel_3d,
            kind: TargetKind::Texture,
            owner_window_id: None,
            size_override: Some(glam::UVec2::new(360, 240)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    ];

    let mut cmds = Vec::new();
    for target in targets {
        cmds.push(EngineCmd::CmdTargetUpsert(target));
    }

    (target_ids, cmds)
}

pub fn build_bind_cmds(targets: Demo006TargetIds, realms: Demo006BindRealms) -> Vec<EngineCmd> {
    let binds = vec![
        CmdTargetBindUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_main,
            layout: TargetBindLayout::default(),
        },
        CmdTargetBindUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.panel_ui,
            layout: bind_layout(Vec4::new(80.0, 60.0, 520.0, 320.0), 2, 1, None, 0),
        },
        CmdTargetBindUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.texture_ui,
            layout: TargetBindLayout::default(),
        },
        CmdTargetBindUpsertArgs {
            realm_id: realms.view,
            target_id: targets.texture_view,
            layout: TargetBindLayout::default(),
        },
        CmdTargetBindUpsertArgs {
            realm_id: realms.ui_panel_3d,
            target_id: targets.texture_panel_3d,
            layout: TargetBindLayout::default(),
        },
    ];

    let mut cmds = Vec::new();
    for bind in binds {
        cmds.push(EngineCmd::CmdTargetBindUpsert(bind));
    }
    cmds
}

fn bind_layout(
    rect: Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<Vec4>,
    input_flags: u32,
) -> TargetBindLayout {
    TargetBindLayout {
        rect,
        z_index,
        blend_mode,
        clip,
        input_flags,
    }
}
