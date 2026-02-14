use glam::Vec4;

use crate::core::cmd::EngineCmd;
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{TargetLayerLayout, TargetKind};

#[derive(Debug, Clone, Copy)]
pub struct Demo006TargetIds {
    pub window_main: u64,
    pub panel_ui: u64,
    pub realm_viewport_ui: u64,
    pub texture_ui_panel_3d: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo006BindRealms {
    pub host_main: u32,
    pub ui: u32,
    pub ui_viewport: u32,
    pub ui_panel_3d: u32,
}

pub fn build_target_cmds(window_main: u32) -> (Demo006TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo006TargetIds {
        window_main: 9200,
        panel_ui: 9201,
        realm_viewport_ui: 9202,
        texture_ui_panel_3d: 9203,
    };

    let targets = vec![
        CmdTargetUpsertArgs {
            target_id: target_ids.window_main,
            kind: TargetKind::Window,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.panel_ui,
            kind: TargetKind::UiPlane,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.realm_viewport_ui,
            kind: TargetKind::RealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(4),
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.texture_ui_panel_3d,
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(glam::UVec2::new(280, 180)),
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
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_main,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.panel_ui,
            layout: bind_layout(
                Vec4::new(0.0, 0.0, 640.0, 720.0),
                1,
                0,
                Some(Vec4::new(0.0, 0.0, 640.0, 720.0)),
                0,
            ),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui_viewport,
            target_id: targets.realm_viewport_ui,
            layout: bind_layout(
                Vec4::new(20.0, 430.0, 600.0, 260.0),
                2,
                0,
                Some(Vec4::new(20.0, 430.0, 600.0, 260.0)),
                0,
            ),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui_panel_3d,
            target_id: targets.texture_ui_panel_3d,
            layout: TargetLayerLayout::default(),
        },
    ];

    let mut cmds = Vec::new();
    for bind in binds {
        cmds.push(EngineCmd::CmdTargetLayerUpsert(bind));
    }
    cmds
}

fn bind_layout(
    rect: Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<Vec4>
) -> TargetLayerLayout {
    TargetLayerLayout {
        rect,
        z_index,
        blend_mode,
        clip
    }
}
