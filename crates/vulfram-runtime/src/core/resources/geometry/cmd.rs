use serde::{Deserialize, Serialize};

use crate::core::render::UniversalGeometryRecord;
use crate::core::resources::vertex::GeometryPrimitiveType;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeometryPrimitiveEntry {
    pub primitive_type: GeometryPrimitiveType,
    pub buffer_id: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGeometryCreateArgs {
    pub geometry_id: u32,
    pub label: Option<String>,
    pub entries: Vec<GeometryPrimitiveEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGeometryUpdateArgs {
    pub geometry_id: u32,
    pub label: Option<String>,
    pub entries: Option<Vec<GeometryPrimitiveEntry>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdGeometryDisposeArgs {
    pub geometry_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryDispose {
    pub success: bool,
    pub message: String,
}

fn validate_entries(entries: &[GeometryPrimitiveEntry]) -> Result<(), String> {
    let has_position = entries
        .iter()
        .any(|e| matches!(e.primitive_type, GeometryPrimitiveType::Position));
    if !has_position {
        return Err("Position primitive is required".into());
    }
    let uv_count = entries
        .iter()
        .filter(|e| matches!(e.primitive_type, GeometryPrimitiveType::UV))
        .count();
    if uv_count > 2 {
        return Err(format!("Too many UV sets (max 2, got {})", uv_count));
    }
    let mut seen_types = std::collections::HashSet::new();
    for entry in entries {
        if !matches!(entry.primitive_type, GeometryPrimitiveType::UV)
            && !seen_types.insert(entry.primitive_type)
        {
            return Err(format!(
                "Duplicate primitive type: {:?}",
                entry.primitive_type
            ));
        }
    }
    Ok(())
}

fn collect_geometry_data(
    engine: &EngineState,
    entries: &[GeometryPrimitiveEntry],
) -> Result<Vec<(GeometryPrimitiveType, Vec<u8>)>, String> {
    let mut data = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(buffer) = engine.buffers.uploads.get(&entry.buffer_id) else {
            return Err(format!("Buffer {} not found", entry.buffer_id));
        };
        data.push((entry.primitive_type, buffer.data.clone()));
    }
    Ok(data)
}

fn consume_entry_buffers(engine: &mut EngineState, entries: &[GeometryPrimitiveEntry]) {
    for entry in entries {
        engine.buffers.uploads.remove(&entry.buffer_id);
    }
}

fn upload_geometry_to_windows(
    engine: &mut EngineState,
    geometry_id: u32,
    label: Option<String>,
    entries: &[(GeometryPrimitiveType, Vec<u8>)],
) {
    for (window_id, render_state) in engine.render.states.iter_mut() {
        let Some(vertex_allocator) = render_state.vertex.as_mut() else {
            continue;
        };
        if vertex_allocator
            .create_geometry(geometry_id, label.clone(), entries.to_vec())
            .is_ok()
        {
            if let Some(window_state) = engine.window.states.get_mut(window_id) {
                window_state.is_dirty = true;
            }
        }
    }
}

pub fn engine_cmd_geometry_create(
    engine: &mut EngineState,
    args: &CmdGeometryCreateArgs,
) -> CmdResultGeometryCreate {
    if let Err(message) = validate_entries(&args.entries) {
        return CmdResultGeometryCreate {
            success: false,
            message,
        };
    }
    let geometry_data = match collect_geometry_data(engine, &args.entries) {
        Ok(data) => data,
        Err(message) => {
            return CmdResultGeometryCreate {
                success: false,
                message,
            };
        }
    };

    engine.universal_state.scene.realm3d.geometries.insert(
        args.geometry_id,
        UniversalGeometryRecord {
            label: args.label.clone(),
            entries: geometry_data.clone(),
        },
    );
    upload_geometry_to_windows(engine, args.geometry_id, args.label.clone(), &geometry_data);
    consume_entry_buffers(engine, &args.entries);
    CmdResultGeometryCreate {
        success: true,
        message: "Geometry created successfully".into(),
    }
}

pub fn engine_cmd_geometry_update(
    engine: &mut EngineState,
    args: &CmdGeometryUpdateArgs,
) -> CmdResultGeometryUpdate {
    if args.entries.is_none() {
        if let Some(record) = engine
            .universal_state
            .scene
            .realm3d
            .geometries
            .get_mut(&args.geometry_id)
        {
            if args.label.is_some() {
                record.label = args.label.clone();
            }
            return CmdResultGeometryUpdate {
                success: true,
                message: "Geometry label updated (no data changed)".into(),
            };
        }
        return CmdResultGeometryUpdate {
            success: false,
            message: format!("Geometry {} not found", args.geometry_id),
        };
    }

    let entries = args.entries.as_ref().expect("checked is_some");
    if let Err(message) = validate_entries(entries) {
        return CmdResultGeometryUpdate {
            success: false,
            message,
        };
    }
    let geometry_data = match collect_geometry_data(engine, entries) {
        Ok(data) => data,
        Err(message) => {
            return CmdResultGeometryUpdate {
                success: false,
                message,
            };
        }
    };
    engine.universal_state.scene.realm3d.geometries.insert(
        args.geometry_id,
        UniversalGeometryRecord {
            label: args.label.clone(),
            entries: geometry_data.clone(),
        },
    );
    upload_geometry_to_windows(engine, args.geometry_id, args.label.clone(), &geometry_data);
    consume_entry_buffers(engine, entries);
    CmdResultGeometryUpdate {
        success: true,
        message: "Geometry updated successfully".into(),
    }
}

pub fn engine_cmd_geometry_dispose(
    engine: &mut EngineState,
    args: &CmdGeometryDisposeArgs,
) -> CmdResultGeometryDispose {
    let removed = engine
        .universal_state
        .scene
        .realm3d
        .geometries
        .remove(&args.geometry_id)
        .is_some();
    let mut window_removed = false;
    for (window_id, render_state) in engine.render.states.iter_mut() {
        let Some(vertex_allocator) = render_state.vertex.as_mut() else {
            continue;
        };
        if vertex_allocator.destroy_geometry(args.geometry_id).is_ok() {
            window_removed = true;
            if let Some(window_state) = engine.window.states.get_mut(window_id) {
                window_state.is_dirty = true;
            }
        }
    }
    if removed || window_removed {
        CmdResultGeometryDispose {
            success: true,
            message: "Geometry disposed successfully".into(),
        }
    } else {
        CmdResultGeometryDispose {
            success: false,
            message: format!("Geometry {} not found", args.geometry_id),
        }
    }
}
