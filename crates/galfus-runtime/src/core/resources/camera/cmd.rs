use glam::{Mat4, Vec2};
use serde::{Deserialize, Serialize};

use crate::core::realm::RealmId;
use crate::core::resources::common::{default_layer_mask, mark_realm_windows_dirty};
use crate::core::resources::{CameraComponent, CameraKind, CameraNode, ViewPosition};
use crate::core::state::EngineState;
use crate::core::system::push_error_event;
use crate::core::target::TargetKind;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCameraCreateArgs {
    pub realm_id: u32,
    pub camera_id: u32,
    pub label: Option<String>,
    pub transform: Mat4,
    pub kind: CameraKind,
    #[serde(default)]
    pub flags: u32,
    pub near_far: Vec2,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default)]
    pub order: i32,
    pub view_position: Option<ViewPosition>,
    #[serde(default = "default_ortho_scale")]
    pub ortho_scale: f32,
}

fn default_ortho_scale() -> f32 {
    10.0
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCameraUpdateArgs {
    pub realm_id: u32,
    pub camera_id: u32,
    pub label: Option<String>,
    pub transform: Option<Mat4>,
    pub kind: Option<CameraKind>,
    pub flags: Option<u32>,
    pub near_far: Option<Vec2>,
    pub layer_mask: Option<u32>,
    pub order: Option<i32>,
    pub view_position: Option<ViewPosition>,
    pub ortho_scale: Option<f32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCamera3dDisposeArgs {
    pub realm_id: u32,
    pub camera_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraDispose {
    pub success: bool,
    pub message: String,
}

fn projection_size_for_realm(engine: &EngineState, realm_id: RealmId) -> (u32, u32) {
    let Some(realm) = engine
        .universal_state
        .composition
        .realms
        .entries
        .get(&realm_id)
    else {
        return (1, 1);
    };
    if let Some(surface_id) = realm.value.output_surface
        && let Some(surface) = engine
            .universal_state
            .composition
            .surfaces
            .entries
            .get(&surface_id)
    {
        return (surface.value.size.x.max(1), surface.value.size.y.max(1));
    }
    let mut chosen_window_id: Option<u32> = None;
    for layer in engine
        .universal_state
        .targets
        .target_layers
        .entries
        .values()
    {
        if layer.realm_id != realm_id.0 {
            continue;
        }
        let Some(target) = engine
            .universal_state
            .targets
            .targets
            .entries
            .get(&layer.target_id)
        else {
            continue;
        };
        if target.kind != TargetKind::Window {
            continue;
        }
        let Some(window_id) = target.window_id else {
            continue;
        };
        match chosen_window_id {
            Some(current_window_id) if current_window_id <= window_id => {}
            _ => chosen_window_id = Some(window_id),
        }
    }
    if let Some(window_id) = chosen_window_id
        && let Some(window) = engine.window.states.get(&window_id)
    {
        return (window.inner_size.x.max(1), window.inner_size.y.max(1));
    }
    (1, 1)
}

fn camera_create_error(
    engine: &mut EngineState,
    message: String,
    command: &str,
) -> CmdResultCameraCreate {
    push_error_event(
        engine,
        "camera",
        message.clone(),
        None,
        Some(command.to_string()),
    );
    CmdResultCameraCreate {
        success: false,
        message,
    }
}

pub fn engine_cmd_camera_create(
    engine: &mut EngineState,
    args: &CmdCameraCreateArgs,
) -> CmdResultCameraCreate {
    let realm_id = RealmId(args.realm_id);
    let projection_size = projection_size_for_realm(engine, realm_id);
    let entities = engine
        .universal_state
        .scene
        .realm3d
        .entities
        .entry(realm_id)
        .or_default();
    if entities.cameras.contains_key(&args.camera_id) {
        return camera_create_error(
            engine,
            format!("Camera with id {} already exists", args.camera_id),
            "camera3d-upsert",
        );
    }

    let data = CameraComponent::new(
        args.transform,
        args.kind,
        args.flags,
        args.near_far,
        projection_size,
        args.ortho_scale,
    );
    entities.cameras.insert(
        args.camera_id,
        CameraNode {
            label: args.label.clone(),
            data,
            layer_mask: args.layer_mask,
            order: args.order,
            ortho_scale: args.ortho_scale,
            view_position: args.view_position.clone(),
        },
    );
    mark_realm_windows_dirty(engine, args.realm_id);
    galfus_log::galfus_log_debug!(
        engine,
        "realm3d.state",
        "camera-created realm={} camera={} kind={:?} near={} far={} layer_mask={} order={}",
        args.realm_id,
        args.camera_id,
        args.kind,
        args.near_far.x,
        args.near_far.y,
        args.layer_mask,
        args.order
    );

    CmdResultCameraCreate {
        success: true,
        message: "Camera created successfully".into(),
    }
}

pub fn engine_cmd_camera_update(
    engine: &mut EngineState,
    args: &CmdCameraUpdateArgs,
) -> CmdResultCameraUpdate {
    let realm_id = RealmId(args.realm_id);
    let projection_size = projection_size_for_realm(engine, realm_id);
    let Some(entities) = engine
        .universal_state
        .scene
        .realm3d
        .entities
        .get_mut(&realm_id)
    else {
        let message = format!("Realm {} not found", args.realm_id);
        push_error_event(
            engine,
            "camera",
            message.clone(),
            None,
            Some("camera3d-upsert".into()),
        );
        return CmdResultCameraUpdate {
            success: false,
            message,
        };
    };
    let Some(camera) = entities.cameras.get_mut(&args.camera_id) else {
        let message = format!("Camera with id {} not found", args.camera_id);
        push_error_event(
            engine,
            "camera",
            message.clone(),
            None,
            Some("camera3d-upsert".into()),
        );
        return CmdResultCameraUpdate {
            success: false,
            message,
        };
    };

    if args.label.is_some() {
        camera.label = args.label.clone();
    }
    if let Some(layer_mask) = args.layer_mask {
        camera.layer_mask = layer_mask;
    }
    if let Some(order) = args.order {
        camera.order = order;
    }
    if let Some(view_position) = args.view_position.clone() {
        camera.view_position = Some(view_position);
    }
    if let Some(ortho_scale) = args.ortho_scale {
        camera.ortho_scale = ortho_scale;
    }
    camera.data.update(
        args.transform,
        args.kind,
        args.flags,
        args.near_far,
        projection_size,
        camera.ortho_scale,
    );
    mark_realm_windows_dirty(engine, args.realm_id);

    CmdResultCameraUpdate {
        success: true,
        message: "Camera updated successfully".into(),
    }
}

pub fn engine_cmd_camera_dispose(
    engine: &mut EngineState,
    args: &CmdCamera3dDisposeArgs,
) -> CmdResultCameraDispose {
    let realm_id = RealmId(args.realm_id);
    let Some(entities) = engine
        .universal_state
        .scene
        .realm3d
        .entities
        .get_mut(&realm_id)
    else {
        let message = format!("Realm {} not found", args.realm_id);
        push_error_event(
            engine,
            "camera",
            message.clone(),
            None,
            Some("camera3d-dispose".into()),
        );
        return CmdResultCameraDispose {
            success: false,
            message,
        };
    };

    if entities.cameras.remove(&args.camera_id).is_some() {
        mark_realm_windows_dirty(engine, args.realm_id);
        CmdResultCameraDispose {
            success: true,
            message: "Camera disposed successfully".into(),
        }
    } else {
        let message = format!("Camera with id {} not found", args.camera_id);
        push_error_event(
            engine,
            "camera",
            message.clone(),
            None,
            Some("camera3d-dispose".into()),
        );
        CmdResultCameraDispose {
            success: false,
            message,
        }
    }
}
