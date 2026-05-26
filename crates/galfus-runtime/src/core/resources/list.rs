use crate::core::id_policy::validate_host_logical_id;
use crate::core::resources::MaterialRealmKind;
use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceEntry {
    pub id: u32,
    pub label: Option<String>,
}

// -----------------------------------------------------------------------------
// List Models
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelList {
    pub success: bool,
    pub message: String,
    pub models: Vec<ResourceEntry>,
}

pub fn engine_cmd_model_list(
    engine: &mut EngineState,
    args: &CmdModelListArgs,
) -> CmdResultModelList {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultModelList {
            success: false,
            message,
            ..Default::default()
        };
    }
    let Some(realm_id) = engine
        .universal_state
        .targets
        .host_realm_index
        .get(&args.window_id)
        .copied()
    else {
        return CmdResultModelList {
            success: false,
            message: format!("No host realm for window {}", args.window_id),
            ..Default::default()
        };
    };
    let Some(entities) = engine.universal_state.scene.realm3d.entities.get(&realm_id) else {
        return CmdResultModelList {
            success: false,
            message: format!("Realm {} not found", realm_id.0),
            ..Default::default()
        };
    };
    let models = entities
        .models
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultModelList {
        success: true,
        message: "Models listed successfully".into(),
        models,
    }
}

// -----------------------------------------------------------------------------
// List Materials
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialListArgs {
    pub window_id: u32,
    pub realm_kind: Option<MaterialRealmKind>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialList {
    pub success: bool,
    pub message: String,
    pub materials: Vec<ResourceEntry>,
}

pub fn engine_cmd_material_list(
    engine: &mut EngineState,
    args: &CmdMaterialListArgs,
) -> CmdResultMaterialList {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultMaterialList {
            success: false,
            message,
            ..Default::default()
        };
    }
    let _ = args;
    let realm3d = &engine.universal_state.scene.realm3d;

    let mut materials = Vec::new();

    for (&id, rec) in &realm3d.materials {
        if let Some(filter_kind) = args.realm_kind
            && rec.realm_kind != filter_kind
        {
            continue;
        }
        materials.push(ResourceEntry {
            id,
            label: rec.label.clone(),
        });
    }

    CmdResultMaterialList {
        success: true,
        message: "Materials listed successfully".into(),
        materials,
    }
}

// -----------------------------------------------------------------------------
// List Textures
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTextureListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureList {
    pub success: bool,
    pub message: String,
    pub textures: Vec<ResourceEntry>,
}

pub fn engine_cmd_texture_list(
    engine: &mut EngineState,
    args: &CmdTextureListArgs,
) -> CmdResultTextureList {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultTextureList {
            success: false,
            message,
            ..Default::default()
        };
    }
    let _ = args;
    let render_resources = &engine.universal_state.scene.render_resources;

    let mut textures = Vec::new();

    for (&id, rec) in &render_resources.textures {
        textures.push(ResourceEntry {
            id,
            label: rec.label.clone(),
        });
    }

    for (&id, entry) in &render_resources.forward_atlas_entries {
        textures.push(ResourceEntry {
            id,
            label: entry.label.clone(),
        });
    }

    for (&id, entry) in &render_resources.target_texture_binds {
        textures.push(ResourceEntry {
            id,
            label: entry.label.clone(),
        });
    }

    CmdResultTextureList {
        success: true,
        message: "Textures listed successfully".into(),
        textures,
    }
}

// -----------------------------------------------------------------------------
// List Geometry (Added for completeness while we're at it)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGeometryListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryList {
    pub success: bool,
    pub message: String,
    pub geometries: Vec<ResourceEntry>,
}

pub fn engine_cmd_geometry_list(
    engine: &mut EngineState,
    args: &CmdGeometryListArgs,
) -> CmdResultGeometryList {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultGeometryList {
            success: false,
            message,
            ..Default::default()
        };
    }
    let _ = args;
    let geometries = engine
        .universal_state
        .scene
        .realm3d
        .geometries
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultGeometryList {
        success: true,
        message: "Geometries listed successfully".into(),
        geometries,
    }
}

// -----------------------------------------------------------------------------
// List Lights
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdLightListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightList {
    pub success: bool,
    pub message: String,
    pub lights: Vec<ResourceEntry>,
}

pub fn engine_cmd_light_list(
    engine: &mut EngineState,
    args: &CmdLightListArgs,
) -> CmdResultLightList {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultLightList {
            success: false,
            message,
            ..Default::default()
        };
    }
    let Some(realm_id) = engine
        .universal_state
        .targets
        .host_realm_index
        .get(&args.window_id)
        .copied()
    else {
        return CmdResultLightList {
            success: false,
            message: format!("No host realm for window {}", args.window_id),
            ..Default::default()
        };
    };
    let Some(entities) = engine.universal_state.scene.realm3d.entities.get(&realm_id) else {
        return CmdResultLightList {
            success: false,
            message: format!("Realm {} not found", realm_id.0),
            ..Default::default()
        };
    };
    let lights = entities
        .lights
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultLightList {
        success: true,
        message: "Lights listed successfully".into(),
        lights,
    }
}

// -----------------------------------------------------------------------------
// List Cameras
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCameraListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraList {
    pub success: bool,
    pub message: String,
    pub cameras: Vec<ResourceEntry>,
}

pub fn engine_cmd_camera_list(
    engine: &mut EngineState,
    args: &CmdCameraListArgs,
) -> CmdResultCameraList {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultCameraList {
            success: false,
            message,
            ..Default::default()
        };
    }
    let Some(realm_id) = engine
        .universal_state
        .targets
        .host_realm_index
        .get(&args.window_id)
        .copied()
    else {
        return CmdResultCameraList {
            success: false,
            message: format!("No host realm for window {}", args.window_id),
            ..Default::default()
        };
    };
    let Some(entities) = engine.universal_state.scene.realm3d.entities.get(&realm_id) else {
        return CmdResultCameraList {
            success: false,
            message: format!("Realm {} not found", realm_id.0),
            ..Default::default()
        };
    };
    let cameras = entities
        .cameras
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultCameraList {
        success: true,
        message: "Cameras listed successfully".into(),
        cameras,
    }
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
