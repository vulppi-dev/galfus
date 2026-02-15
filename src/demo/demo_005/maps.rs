use glam::Vec4;

use crate::core::cmd::EngineCmd;
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};

#[derive(Debug, Clone, Copy)]
pub struct Demo005TargetIds {
    pub window_main: u64,
    pub window_aux: u64,
    pub window_layer_main: u64,
    pub window_layer_aux: u64,
    pub realm_plane_layer: u64,
    pub texture_shared: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo005LayerRealms {
    pub host_main: u32,
    pub host_aux: u32,
    pub window_layer_main: u32,
    pub ui: u32,
    pub texture_main: u32,
    pub texture_aux: u32,
    pub conflict: u32,
}

pub fn build_target_cmds(window_main: u32, window_aux: u32) -> (Demo005TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo005TargetIds {
        window_main: 9000,
        window_aux: 9001,
        window_layer_main: 9002,
        realm_plane_layer: 9003,
        window_layer_aux: 9004,
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
            target_id: target_ids.window_layer_main,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(4),
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.realm_plane_layer,
            kind: TargetKind::RealmPlane,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.window_layer_aux,
            kind: TargetKind::WidgetRealmViewport,
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

pub fn build_layer_cmds(targets: Demo005TargetIds, realms: Demo005LayerRealms) -> Vec<EngineCmd> {
    let layers = vec![
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_main,
            layout: TargetLayerLayout::default(),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_aux,
            target_id: targets.window_aux,
            layout: TargetLayerLayout::default(),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.window_layer_main,
            target_id: targets.window_layer_main,
            layout: layer_layout(Vec4::new(40.0, 40.0, 320.0, 220.0), 2, 0, None),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.realm_plane_layer,
            layout: layer_layout(
                Vec4::new(720.0, 120.0, 220.0, 180.0),
                3,
                1,
                Some(Vec4::new(720.0, 120.0, 160.0, 140.0)),
            ),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.texture_main,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.texture_aux,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.conflict,
            target_id: targets.window_layer_main,
            layout: TargetLayerLayout::default(),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.conflict,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_layer_main,
            layout: layer_layout(Vec4::new(60.0, 360.0, 220.0, 160.0), 1, 0, None),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_layer_aux,
            layout: layer_layout(Vec4::new(1020.0, 40.0, 180.0, 120.0), 0, 0, None),
            camera_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_aux,
            target_id: targets.window_layer_main,
            layout: layer_layout(Vec4::new(20.0, 40.0, 200.0, 140.0), 0, 0, None),
            camera_id: None,
        },
    ];

    let mut cmds = Vec::new();
    for layer in layers {
        cmds.push(EngineCmd::CmdTargetLayerUpsert(layer));
    }
    cmds
}

fn layer_layout(
    rect: Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<Vec4>,
) -> TargetLayerLayout {
    TargetLayerLayout {
        left: DimensionValue::Px(rect.x),
        top: DimensionValue::Px(rect.y),
        width: DimensionValue::Px(rect.z),
        height: DimensionValue::Px(rect.w),
        z_index,
        blend_mode,
        clip,
    }
}
