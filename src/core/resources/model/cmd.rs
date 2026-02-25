use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::render::state::SkinningSystem;
use crate::core::resources::common::{default_layer_mask, mark_realm_windows_dirty};
use crate::core::resources::{ModelComponent, ModelRecord};
use crate::core::state::EngineState;
use crate::core::system::push_error_event;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelCreateArgs {
    pub realm_id: u32,
    pub model_id: u32,
    pub label: Option<String>,
    pub geometry_id: u32,
    #[serde(default)]
    pub material_id: Option<u32>,
    pub transform: Mat4,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub cast_shadow: bool,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub receive_shadow: bool,
    #[serde(default)]
    pub cast_outline: bool,
    #[serde(default = "crate::core::resources::common::default_vec4_zero")]
    pub outline_color: glam::Vec4,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelUpdateArgs {
    pub realm_id: u32,
    pub model_id: u32,
    pub label: Option<String>,
    pub geometry_id: Option<u32>,
    #[serde(default)]
    pub material_id: Option<u32>,
    pub transform: Option<Mat4>,
    pub layer_mask: Option<u32>,
    pub cast_shadow: Option<bool>,
    pub receive_shadow: Option<bool>,
    pub cast_outline: Option<bool>,
    pub outline_color: Option<glam::Vec4>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdPoseUpdateArgs {
    pub realm_id: u32,
    pub model_id: u32,
    pub bone_count: u32,
    pub matrices_buffer_id: u64,
    #[serde(default)]
    pub window_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultPoseUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdModelDisposeArgs {
    pub realm_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelDispose {
    pub success: bool,
    pub message: String,
}

fn fail_model(
    engine: &mut EngineState,
    scope: &str,
    command: &str,
    message: String,
) -> CmdResultModelUpdate {
    push_error_event(
        engine,
        scope,
        message.clone(),
        None,
        Some(command.to_string()),
    );
    CmdResultModelUpdate {
        success: false,
        message,
    }
}

pub fn engine_cmd_model_create(
    engine: &mut EngineState,
    args: &CmdModelCreateArgs,
) -> CmdResultModelCreate {
    let realm_id = crate::core::realm::RealmId(args.realm_id);
    if !engine
        .universal_state
        .realms
        .entries
        .contains_key(&realm_id)
    {
        let message = format!("Realm {} not found", args.realm_id);
        push_error_event(
            engine,
            "model",
            message.clone(),
            None,
            Some("model-upsert".into()),
        );
        return CmdResultModelCreate {
            success: false,
            message,
        };
    }
    let entities = engine
        .universal_state
        .realm_entities
        .entry(realm_id)
        .or_default();
    if entities.models.contains_key(&args.model_id) {
        let message = format!("Model with id {} already exists", args.model_id);
        push_error_event(
            engine,
            "model",
            message.clone(),
            None,
            Some("model-upsert".into()),
        );
        return CmdResultModelCreate {
            success: false,
            message,
        };
    }

    let component = ModelComponent::new(args.transform, args.receive_shadow, args.outline_color);
    let record = ModelRecord::new(
        args.label.clone(),
        component,
        args.geometry_id,
        args.material_id,
        args.layer_mask,
        args.cast_shadow,
        args.receive_shadow,
        args.cast_outline,
    );
    entities.models.insert(args.model_id, record);
    mark_realm_windows_dirty(engine, args.realm_id);

    CmdResultModelCreate {
        success: true,
        message: "Model created successfully".into(),
    }
}

pub fn engine_cmd_model_update(
    engine: &mut EngineState,
    args: &CmdModelUpdateArgs,
) -> CmdResultModelUpdate {
    let realm_id = crate::core::realm::RealmId(args.realm_id);
    let Some(entities) = engine.universal_state.realm_entities.get_mut(&realm_id) else {
        return fail_model(
            engine,
            "model",
            "model-upsert",
            format!("Realm {} not found", args.realm_id),
        );
    };

    let Some(record) = entities.models.get_mut(&args.model_id) else {
        return fail_model(
            engine,
            "model",
            "model-upsert",
            format!("Model with id {} not found", args.model_id),
        );
    };

    if args.label.is_some() {
        record.label = args.label.clone();
    }
    if let Some(geometry_id) = args.geometry_id {
        record.geometry_id = geometry_id;
    }
    if args.material_id.is_some() {
        record.material_id = args.material_id;
    }
    if let Some(cast_shadow) = args.cast_shadow {
        record.cast_shadow = cast_shadow;
    }
    if let Some(receive_shadow) = args.receive_shadow {
        record.receive_shadow = receive_shadow;
    }
    if let Some(cast_outline) = args.cast_outline {
        record.cast_outline = cast_outline;
    }
    record
        .data
        .update(args.transform, args.receive_shadow, args.outline_color);
    if let Some(layer_mask) = args.layer_mask {
        record.layer_mask = layer_mask;
    }
    record.mark_dirty();
    mark_realm_windows_dirty(engine, args.realm_id);

    CmdResultModelUpdate {
        success: true,
        message: "Model updated successfully".into(),
    }
}

pub fn engine_cmd_pose_update(
    engine: &mut EngineState,
    args: &CmdPoseUpdateArgs,
) -> CmdResultPoseUpdate {
    let realm_id = crate::core::realm::RealmId(args.realm_id);
    let Some(entities) = engine.universal_state.realm_entities.get_mut(&realm_id) else {
        let message = format!("Realm {} not found", args.realm_id);
        push_error_event(
            engine,
            "pose",
            message.clone(),
            None,
            Some("pose-update".into()),
        );
        return CmdResultPoseUpdate {
            success: false,
            message,
        };
    };
    let Some(model) = entities.models.get_mut(&args.model_id) else {
        let message = format!("Model with id {} not found", args.model_id);
        push_error_event(
            engine,
            "pose",
            message.clone(),
            None,
            Some("pose-update".into()),
        );
        return CmdResultPoseUpdate {
            success: false,
            message,
        };
    };
    let Some(buffer) = engine.buffers.uploads.get(&args.matrices_buffer_id) else {
        let message = format!("Buffer {} not found", args.matrices_buffer_id);
        push_error_event(
            engine,
            "pose",
            message.clone(),
            None,
            Some("pose-update".into()),
        );
        return CmdResultPoseUpdate {
            success: false,
            message,
        };
    };
    if args.bone_count == 0 {
        return CmdResultPoseUpdate {
            success: false,
            message: "boneCount must be greater than 0".into(),
        };
    }
    if args.bone_count > SkinningSystem::MAX_BONES_PER_MODEL {
        return CmdResultPoseUpdate {
            success: false,
            message: format!(
                "boneCount exceeds limit (max {})",
                SkinningSystem::MAX_BONES_PER_MODEL
            ),
        };
    }
    let expected_bytes = args.bone_count as usize * std::mem::size_of::<glam::Mat4>();
    if buffer.data.len() != expected_bytes {
        return CmdResultPoseUpdate {
            success: false,
            message: format!(
                "Bone matrix buffer size mismatch (expected {} bytes, got {})",
                expected_bytes,
                buffer.data.len()
            ),
        };
    }

    let matrices: &[glam::Mat4] = bytemuck::cast_slice(&buffer.data);
    let mut applied = false;
    let mut first_offset = None;
    for (&window_id, render_state) in engine.render.states.iter_mut() {
        if args
            .window_id
            .is_some_and(|requested| requested != window_id)
        {
            continue;
        }
        let Some(vertex_allocator) = render_state.vertex.as_ref() else {
            continue;
        };
        let has_skin_streams = vertex_allocator
            .geometry_has_streams(
                model.geometry_id,
                &[
                    crate::core::resources::vertex::VertexStream::Joints,
                    crate::core::resources::vertex::VertexStream::Weights,
                ],
            )
            .unwrap_or(false);
        if !has_skin_streams {
            continue;
        }
        let Some(bindings) = render_state.bindings.as_mut() else {
            continue;
        };
        let allocation = render_state
            .skinning
            .ensure_allocation(args.model_id, args.bone_count);
        bindings.bones_pool.write_slice(allocation.offset, matrices);
        first_offset.get_or_insert(allocation.offset);
        applied = true;
        if let Some(window_state) = engine.window.states.get_mut(&window_id) {
            window_state.is_dirty = true;
        }
    }
    let _ = engine.buffers.remove_upload(args.matrices_buffer_id);
    if applied {
        model
            .data
            .set_skinning(first_offset.unwrap_or(0), args.bone_count);
        model.mark_dirty();
        CmdResultPoseUpdate {
            success: true,
            message: "Pose updated successfully".into(),
        }
    } else {
        let message = "No compatible render state to apply skinning".to_string();
        push_error_event(
            engine,
            "pose",
            message.clone(),
            None,
            Some("pose-update".into()),
        );
        CmdResultPoseUpdate {
            success: false,
            message,
        }
    }
}

pub fn engine_cmd_model_dispose(
    engine: &mut EngineState,
    args: &CmdModelDisposeArgs,
) -> CmdResultModelDispose {
    let realm_id = crate::core::realm::RealmId(args.realm_id);
    let Some(entities) = engine.universal_state.realm_entities.get_mut(&realm_id) else {
        let message = format!("Realm {} not found", args.realm_id);
        push_error_event(
            engine,
            "model",
            message.clone(),
            None,
            Some("model-dispose".into()),
        );
        return CmdResultModelDispose {
            success: false,
            message,
        };
    };

    if entities.models.remove(&args.model_id).is_some() {
        for render_state in engine.render.states.values_mut() {
            render_state.skinning.release(args.model_id);
        }
        mark_realm_windows_dirty(engine, args.realm_id);
        CmdResultModelDispose {
            success: true,
            message: "Model disposed successfully".into(),
        }
    } else {
        let message = format!("Model with id {} not found", args.model_id);
        push_error_event(
            engine,
            "model",
            message.clone(),
            None,
            Some("model-dispose".into()),
        );
        CmdResultModelDispose {
            success: false,
            message,
        }
    }
}
