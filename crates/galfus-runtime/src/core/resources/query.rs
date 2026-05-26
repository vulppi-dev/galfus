use crate::core::id_policy::validate_host_logical_id;
use crate::core::realm::RealmId;
use crate::core::resources::MaterialRealmKind;
use crate::core::resources::list::ResourceEntry;
use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct QueryScopeArgs {
    pub window_id: Option<u32>,
    pub realm_id: Option<u32>,
    pub ids: Option<Vec<u32>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultResourceGet {
    pub success: bool,
    pub message: String,
    pub kind: String,
    pub id: Option<u32>,
    pub label: Option<String>,
    pub realm_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultResourceList {
    pub success: bool,
    pub message: String,
    pub kind: String,
    pub items: Vec<ResourceEntry>,
}

fn resolve_realm_scope(engine: &EngineState, scope: &QueryScopeArgs) -> Option<RealmId> {
    if let Some(realm_id) = scope.realm_id {
        return Some(RealmId(realm_id));
    }
    scope.window_id.and_then(|window_id| {
        engine
            .universal_state
            .targets
            .host_realm_index
            .get(&window_id)
            .copied()
    })
}

fn validate_scope_ids(scope: &QueryScopeArgs) -> Result<(), String> {
    if let Some(window_id) = scope.window_id {
        validate_host_logical_id(window_id, "windowId")?;
    }
    if let Some(realm_id) = scope.realm_id {
        validate_host_logical_id(realm_id, "realmId")?;
    }
    if let Some(ids) = scope.ids.as_ref() {
        for id in ids {
            validate_host_logical_id(*id, "id")?;
        }
    }
    Ok(())
}

fn id_allowed(scope: &QueryScopeArgs, id: u32) -> bool {
    scope.ids.as_ref().is_none_or(|ids| ids.contains(&id))
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResourceGetArgs {
    pub id: u32,
    pub scope: QueryScopeArgs,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialGetArgs {
    pub id: u32,
    pub scope: QueryScopeArgs,
    pub realm_kind: Option<MaterialRealmKind>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialInstanceGetArgs {
    pub id: u32,
    pub scope: QueryScopeArgs,
    pub realm_kind: Option<MaterialRealmKind>,
}

pub fn engine_cmd_camera_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "camera".into(),
            ..Default::default()
        };
    }
    let Some(realm_id) = resolve_realm_scope(engine, &args.scope) else {
        return CmdResultResourceGet {
            success: false,
            message: "Realm scope not resolved".into(),
            kind: "camera".into(),
            ..Default::default()
        };
    };
    let Some(entities) = engine.universal_state.scene.realm3d.entities.get(&realm_id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Realm not found".into(),
            kind: "camera".into(),
            ..Default::default()
        };
    };
    let Some(rec) = entities.cameras.get(&args.id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Camera not found".into(),
            kind: "camera".into(),
            ..Default::default()
        };
    };
    CmdResultResourceGet {
        success: true,
        message: "Camera found".into(),
        kind: "camera".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        realm_id: Some(realm_id.0),
    }
}

pub fn engine_cmd_model_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "model".into(),
            ..Default::default()
        };
    }
    let Some(realm_id) = resolve_realm_scope(engine, &args.scope) else {
        return CmdResultResourceGet {
            success: false,
            message: "Realm scope not resolved".into(),
            kind: "model".into(),
            ..Default::default()
        };
    };
    let Some(entities) = engine.universal_state.scene.realm3d.entities.get(&realm_id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Realm not found".into(),
            kind: "model".into(),
            ..Default::default()
        };
    };
    let Some(rec) = entities.models.get(&args.id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Model not found".into(),
            kind: "model".into(),
            ..Default::default()
        };
    };
    CmdResultResourceGet {
        success: true,
        message: "Model found".into(),
        kind: "model".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        realm_id: Some(realm_id.0),
    }
}

pub fn engine_cmd_light_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "light".into(),
            ..Default::default()
        };
    }
    let Some(realm_id) = resolve_realm_scope(engine, &args.scope) else {
        return CmdResultResourceGet {
            success: false,
            message: "Realm scope not resolved".into(),
            kind: "light".into(),
            ..Default::default()
        };
    };
    let Some(entities) = engine.universal_state.scene.realm3d.entities.get(&realm_id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Realm not found".into(),
            kind: "light".into(),
            ..Default::default()
        };
    };
    let Some(rec) = entities.lights.get(&args.id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Light not found".into(),
            kind: "light".into(),
            ..Default::default()
        };
    };
    CmdResultResourceGet {
        success: true,
        message: "Light found".into(),
        kind: "light".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        realm_id: Some(realm_id.0),
    }
}

pub fn engine_cmd_geometry_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "geometry".into(),
            ..Default::default()
        };
    }
    let Some(rec) = engine
        .universal_state
        .scene
        .realm3d
        .geometries
        .get(&args.id)
    else {
        return CmdResultResourceGet {
            success: false,
            message: "Geometry not found".into(),
            kind: "geometry".into(),
            ..Default::default()
        };
    };
    CmdResultResourceGet {
        success: true,
        message: "Geometry found".into(),
        kind: "geometry".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        ..Default::default()
    }
}

pub fn engine_cmd_material_get(
    engine: &mut EngineState,
    args: &CmdMaterialGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "material".into(),
            ..Default::default()
        };
    }
    let Some(rec) = engine.universal_state.scene.realm3d.materials.get(&args.id) else {
        return CmdResultResourceGet {
            success: false,
            message: "Material not found".into(),
            kind: "material".into(),
            ..Default::default()
        };
    };
    if let Some(filter_kind) = args.realm_kind
        && rec.realm_kind != filter_kind
    {
        return CmdResultResourceGet {
            success: false,
            message: "Material realm kind mismatch".into(),
            kind: "material".into(),
            ..Default::default()
        };
    }
    CmdResultResourceGet {
        success: true,
        message: "Material found".into(),
        kind: "material".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        ..Default::default()
    }
}

pub fn engine_cmd_texture_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "texture".into(),
            ..Default::default()
        };
    }
    if let Some(rec) = engine
        .universal_state
        .scene
        .render_resources
        .textures
        .get(&args.id)
    {
        return CmdResultResourceGet {
            success: true,
            message: "Texture found".into(),
            kind: "texture".into(),
            id: Some(args.id),
            label: rec.label.clone(),
            ..Default::default()
        };
    }
    if let Some(rec) = engine
        .universal_state
        .scene
        .render_resources
        .forward_atlas_entries
        .get(&args.id)
    {
        return CmdResultResourceGet {
            success: true,
            message: "Texture atlas entry found".into(),
            kind: "texture".into(),
            id: Some(args.id),
            label: rec.label.clone(),
            ..Default::default()
        };
    }
    if let Some(rec) = engine
        .universal_state
        .scene
        .render_resources
        .target_texture_binds
        .get(&args.id)
    {
        return CmdResultResourceGet {
            success: true,
            message: "Texture target bind found".into(),
            kind: "texture".into(),
            id: Some(args.id),
            label: rec.label.clone(),
            ..Default::default()
        };
    }
    CmdResultResourceGet {
        success: false,
        message: "Texture not found".into(),
        kind: "texture".into(),
        ..Default::default()
    }
}

pub fn engine_cmd_environment_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "environment".into(),
            ..Default::default()
        };
    }
    let Some(rec) = engine
        .universal_state
        .scene
        .realm3d
        .environment_profiles
        .get(&args.id)
    else {
        return CmdResultResourceGet {
            success: false,
            message: "Environment not found".into(),
            kind: "environment".into(),
            ..Default::default()
        };
    };
    let _ = rec;
    CmdResultResourceGet {
        success: true,
        message: "Environment found".into(),
        kind: "environment".into(),
        id: Some(args.id),
        label: None,
        ..Default::default()
    }
}

pub fn engine_cmd_material_definition_get(
    engine: &mut EngineState,
    args: &CmdResourceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "material-definition".into(),
            ..Default::default()
        };
    }
    let Some(rec) = engine
        .universal_state
        .scene
        .material_definitions
        .get(&args.id)
    else {
        return CmdResultResourceGet {
            success: false,
            message: "Material definition not found".into(),
            kind: "material-definition".into(),
            ..Default::default()
        };
    };
    CmdResultResourceGet {
        success: true,
        message: "Material definition found".into(),
        kind: "material-definition".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        ..Default::default()
    }
}

pub fn engine_cmd_material_instance_get(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceGetArgs,
) -> CmdResultResourceGet {
    if let Err(message) =
        validate_host_logical_id(args.id, "id").and_then(|_| validate_scope_ids(&args.scope))
    {
        return CmdResultResourceGet {
            success: false,
            message,
            kind: "material-instance".into(),
            ..Default::default()
        };
    }
    let Some(rec) = engine
        .universal_state
        .scene
        .material_instances
        .get(&args.id)
    else {
        return CmdResultResourceGet {
            success: false,
            message: "Material instance not found".into(),
            kind: "material-instance".into(),
            ..Default::default()
        };
    };
    if let Some(filter_kind) = args.realm_kind
        && let Some(material) = engine.universal_state.scene.realm3d.materials.get(&args.id)
        && material.realm_kind != filter_kind
    {
        return CmdResultResourceGet {
            success: false,
            message: "Material instance realm kind mismatch".into(),
            kind: "material-instance".into(),
            ..Default::default()
        };
    }
    CmdResultResourceGet {
        success: true,
        message: "Material instance found".into(),
        kind: "material-instance".into(),
        id: Some(args.id),
        label: rec.label.clone(),
        ..Default::default()
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResourceListArgs {
    pub scope: QueryScopeArgs,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialInstanceListArgs {
    pub scope: QueryScopeArgs,
    pub realm_kind: Option<MaterialRealmKind>,
}

fn entries_from_iter<'a>(
    iter: impl Iterator<Item = (u32, Option<String>)> + 'a,
    scope: &QueryScopeArgs,
) -> Vec<ResourceEntry> {
    iter.filter(|(id, _)| id_allowed(scope, *id))
        .map(|(id, label)| ResourceEntry { id, label })
        .collect()
}

pub fn engine_cmd_environment_list(
    engine: &mut EngineState,
    args: &CmdResourceListArgs,
) -> CmdResultResourceList {
    if let Err(message) = validate_scope_ids(&args.scope) {
        return CmdResultResourceList {
            success: false,
            message,
            kind: "environment".into(),
            items: Vec::new(),
        };
    }
    let items = entries_from_iter(
        engine
            .universal_state
            .scene
            .realm3d
            .environment_profiles
            .iter()
            .map(|(&id, _rec)| (id, None)),
        &args.scope,
    );
    CmdResultResourceList {
        success: true,
        message: "Environments listed".into(),
        kind: "environment".into(),
        items,
    }
}

pub fn engine_cmd_material_definition_list(
    engine: &mut EngineState,
    args: &CmdResourceListArgs,
) -> CmdResultResourceList {
    if let Err(message) = validate_scope_ids(&args.scope) {
        return CmdResultResourceList {
            success: false,
            message,
            kind: "material-definition".into(),
            items: Vec::new(),
        };
    }
    let items = entries_from_iter(
        engine
            .universal_state
            .scene
            .material_definitions
            .iter()
            .map(|(&id, rec)| (id, rec.label.clone())),
        &args.scope,
    );
    CmdResultResourceList {
        success: true,
        message: "Material definitions listed".into(),
        kind: "material-definition".into(),
        items,
    }
}

pub fn engine_cmd_material_instance_list(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceListArgs,
) -> CmdResultResourceList {
    if let Err(message) = validate_scope_ids(&args.scope) {
        return CmdResultResourceList {
            success: false,
            message,
            kind: "material-instance".into(),
            items: Vec::new(),
        };
    }
    let items = entries_from_iter(
        engine
            .universal_state
            .scene
            .material_instances
            .iter()
            .filter(|(id, _)| {
                if let Some(filter_kind) = args.realm_kind {
                    if let Some(material) = engine.universal_state.scene.realm3d.materials.get(id) {
                        return material.realm_kind == filter_kind;
                    }
                    return false;
                }
                true
            })
            .map(|(&id, rec)| (id, rec.label.clone())),
        &args.scope,
    );
    CmdResultResourceList {
        success: true,
        message: "Material instances listed".into(),
        kind: "material-instance".into(),
        items,
    }
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
