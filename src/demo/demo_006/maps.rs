use crate::core::cmd::EngineCmd;
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};

#[derive(Debug, Clone, Copy)]
pub struct Demo006TargetIds {
    pub window_main: u64,
    pub realm_plane_layer: u64,
    pub widget_realm_viewport: u64,
    pub texture_ui_panel_3d: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo006LayerRealms {
    pub host_main: u32,
    pub ui: u32,
    pub realm_3d_embed: u32,
    pub ui_panel_3d: u32,
}

pub fn build_target_cmds(window_main: u32) -> (Demo006TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo006TargetIds {
        window_main: 9200,
        realm_plane_layer: 9201,
        widget_realm_viewport: 9202,
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
            target_id: target_ids.realm_plane_layer,
            kind: TargetKind::RealmPlane,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.widget_realm_viewport,
            kind: TargetKind::WidgetRealmViewport,
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

pub fn build_layer_cmds(targets: Demo006TargetIds, realms: Demo006LayerRealms) -> Vec<EngineCmd> {
    let layers = vec![
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_main,
            layout: TargetLayerLayout::default(),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.realm_plane_layer,
            layout: TargetLayerLayout {
                left: DimensionValue::Px(0.0),
                top: DimensionValue::Px(0.0),
                width: DimensionValue::Px(360.0),
                height: DimensionValue::Percent(100.0),
                z_index: 1,
                blend_mode: 0,
                clip: None,
            },
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.realm_3d_embed,
            target_id: targets.widget_realm_viewport,
            layout: TargetLayerLayout {
                left: DimensionValue::Px(20.0),
                top: DimensionValue::Px(430.0),
                width: DimensionValue::Px(320.0),
                height: DimensionValue::Px(240.0),
                z_index: 2,
                blend_mode: 0,
                clip: None,
            },
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui_panel_3d,
            target_id: targets.texture_ui_panel_3d,
            layout: TargetLayerLayout::default(),
        },
    ];

    let mut cmds = Vec::new();
    for layer in layers {
        cmds.push(EngineCmd::CmdTargetLayerUpsert(layer));
    }
    cmds
}
