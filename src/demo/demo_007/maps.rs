use crate::core::cmd::EngineCmd;
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};

pub const ENV_PROFILE_BIND_B_MSAA4: u32 = 9711;
pub const ENV_PROFILE_BIND_C_PROCEDURAL: u32 = 9712;
pub const ENV_PROFILE_BIND_D_PURPLE_CLEAR: u32 = 9713;

#[derive(Debug, Clone, Copy)]
pub struct Demo007TargetIds {
    pub window_main: u64,
    pub widget_view_a: u64,
    pub widget_view_b: u64,
    pub widget_view_c: u64,
    pub widget_view_d: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo007LayerRealms {
    pub ui: u32,
    pub realm_3d: u32,
}

pub fn build_target_cmds(window_main: u32) -> (Demo007TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo007TargetIds {
        window_main: 9700,
        widget_view_a: 9701,
        widget_view_b: 9702,
        widget_view_c: 9703,
        widget_view_d: 9704,
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
            target_id: target_ids.widget_view_a,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(1),
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.widget_view_b,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(1),
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.widget_view_c,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(1),
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.widget_view_d,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(1),
        },
    ];

    let mut cmds = Vec::new();
    for target in targets {
        cmds.push(EngineCmd::CmdTargetUpsert(target));
    }

    (target_ids, cmds)
}

pub fn build_layer_cmds(targets: Demo007TargetIds, realms: Demo007LayerRealms) -> Vec<EngineCmd> {
    let layers = vec![
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.window_main,
            layout: TargetLayerLayout {
                left: DimensionValue::Px(0.0),
                top: DimensionValue::Px(0.0),
                width: DimensionValue::Percent(100.0),
                height: DimensionValue::Percent(100.0),
                z_index: 5,
                blend_mode: 0,
                clip: None,
            },
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.realm_3d,
            target_id: targets.widget_view_a,
            layout: TargetLayerLayout::default(),
            camera_id: Some(7101),
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.realm_3d,
            target_id: targets.widget_view_b,
            layout: TargetLayerLayout::default(),
            camera_id: Some(7102),
            environment_id: Some(ENV_PROFILE_BIND_B_MSAA4),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.realm_3d,
            target_id: targets.widget_view_c,
            layout: TargetLayerLayout::default(),
            camera_id: Some(7103),
            environment_id: Some(ENV_PROFILE_BIND_C_PROCEDURAL),
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.realm_3d,
            target_id: targets.widget_view_d,
            layout: TargetLayerLayout::default(),
            camera_id: Some(7104),
            environment_id: Some(ENV_PROFILE_BIND_D_PURPLE_CLEAR),
        },
    ];

    let mut cmds = Vec::new();
    for layer in layers {
        cmds.push(EngineCmd::CmdTargetLayerUpsert(layer));
    }
    cmds
}
