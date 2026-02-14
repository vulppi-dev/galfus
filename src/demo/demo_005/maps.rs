use glam::Vec4;

use crate::core::cmd::EngineCmd;
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{TargetLayerLayout, TargetKind};

const INPUT_FLAG_RAYCAST: u32 = 1 << 0;

#[derive(Debug, Clone, Copy)]
pub struct Demo005TargetIds {
    pub window_main: u64,
    pub window_aux: u64,
    pub viewport_main: u64,
    pub viewport_aux: u64,
    pub panel_ui: u64,
    pub texture_shared: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo005BindRealms {
    pub host_main: u32,
    pub host_aux: u32,
    pub viewport_main: u32,
    pub ui: u32,
    pub texture_main: u32,
    pub texture_aux: u32,
    pub conflict: u32,
}

pub fn build_target_cmds(window_main: u32, window_aux: u32) -> (Demo005TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo005TargetIds {
        window_main: 9000,
        window_aux: 9001,
        viewport_main: 9002,
        panel_ui: 9003,
        viewport_aux: 9004,
        texture_shared: 9005,
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
            target_id: target_ids.window_aux,
            kind: TargetKind::Window,
            window_id: Some(window_aux),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.viewport_main,
            kind: TargetKind::RealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(4),
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
            target_id: target_ids.viewport_aux,
            kind: TargetKind::RealmViewport,
            window_id: Some(window_aux),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.texture_shared,
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(glam::UVec2::new(256, 256)),
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

pub fn build_bind_cmds(targets: Demo005TargetIds, realms: Demo005BindRealms) -> Vec<EngineCmd> {
    let binds = vec![
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_main,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_aux,
            target_id: targets.window_aux,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.viewport_main,
            target_id: targets.viewport_main,
            layout: bind_layout(
                Vec4::new(40.0, 40.0, 320.0, 220.0),
                2,
                0,
                None,
                INPUT_FLAG_RAYCAST,
            ),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.panel_ui,
            layout: bind_layout(
                Vec4::new(720.0, 120.0, 220.0, 180.0),
                3,
                1,
                Some(Vec4::new(720.0, 120.0, 160.0, 140.0)),
                0,
            ),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.texture_main,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.texture_aux,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.conflict,
            target_id: targets.viewport_main,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.conflict,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.viewport_main,
            layout: bind_layout(Vec4::new(60.0, 360.0, 220.0, 160.0), 1, 0, None, 0),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.viewport_aux,
            layout: bind_layout(Vec4::new(1020.0, 40.0, 180.0, 120.0), 0, 0, None, 0),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_aux,
            target_id: targets.viewport_main,
            layout: bind_layout(Vec4::new(20.0, 40.0, 200.0, 140.0), 0, 0, None),
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
