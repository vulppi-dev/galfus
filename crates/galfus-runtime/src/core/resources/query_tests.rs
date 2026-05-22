use super::*;
use crate::core::realm::RealmId;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdMaterialCreateArgs,
    CmdMaterialDefinitionCreateArgs, CmdMaterialInstanceCreateArgs, EnvironmentConfig,
    MaterialKind, MaterialRealmKind, ShaderMaterialPreset, engine_cmd_camera_create,
    engine_cmd_environment_create, engine_cmd_material_create,
    engine_cmd_material_definition_create, engine_cmd_material_instance_create,
};
use crate::core::test_support::test_engine;
use glam::{Mat4, Vec2};

#[test]
fn camera_get_returns_not_found_when_scope_cannot_be_resolved() {
    let mut engine = test_engine();
    let result = engine_cmd_camera_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 1,
            scope: QueryScopeArgs::default(),
        },
    );

    assert!(!result.success);
    assert_eq!(result.kind, "camera");
    assert_eq!(result.message, "Realm scope not resolved");
}

#[test]
fn camera_get_returns_entity_when_realm_scope_exists() {
    let mut engine = test_engine();
    let realm_id = RealmId(7);
    engine
        .universal_state
        .targets
        .host_realm_index
        .insert(99, realm_id);
    let created = engine_cmd_camera_create(
        &mut engine,
        &CmdCameraCreateArgs {
            realm_id: 7,
            camera_id: 10,
            label: Some("cam-10".into()),
            transform: Mat4::IDENTITY,
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 1000.0),
            layer_mask: u32::MAX,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        },
    );
    assert!(created.success, "{}", created.message);

    let result = engine_cmd_camera_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 10,
            scope: QueryScopeArgs {
                window_id: Some(99),
                ..Default::default()
            },
        },
    );

    assert!(result.success);
    assert_eq!(result.kind, "camera");
    assert_eq!(result.id, Some(10));
    assert_eq!(result.realm_id, Some(7));
}

#[test]
fn environment_get_and_list_support_ids_filter() {
    let mut engine = test_engine();

    assert!(
        engine_cmd_environment_create(
            &mut engine,
            &CmdEnvironmentCreateArgs {
                environment_id: 11,
                config: EnvironmentConfig::default(),
            },
        )
        .success
    );
    assert!(
        engine_cmd_environment_create(
            &mut engine,
            &CmdEnvironmentCreateArgs {
                environment_id: 12,
                config: EnvironmentConfig::default(),
            },
        )
        .success
    );

    let get = engine_cmd_environment_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 11,
            scope: QueryScopeArgs::default(),
        },
    );
    assert!(get.success);
    assert_eq!(get.kind, "environment");
    assert_eq!(get.id, Some(11));

    let listed = engine_cmd_environment_list(
        &mut engine,
        &CmdResourceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![12]),
                ..Default::default()
            },
        },
    );
    assert!(listed.success);
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].id, 12);
}

#[test]
fn material_definition_and_instance_get_and_list_work() {
    let mut engine = test_engine();

    let definition_result = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 901,
            slug: "test-def-901".into(),
            label: Some("Test Definition".into()),
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(definition_result.success, "{}", definition_result.message);

    let instance_result = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 902,
            slug: "test-def-901".into(),
            label: Some("Test Instance".into()),
            options: None,
        },
    );
    assert!(instance_result.success, "{}", instance_result.message);

    let definition_get = engine_cmd_material_definition_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 901,
            scope: QueryScopeArgs::default(),
        },
    );
    assert!(definition_get.success);
    assert_eq!(definition_get.label.as_deref(), Some("Test Definition"));

    let instance_get = engine_cmd_material_instance_get(
        &mut engine,
        &CmdMaterialInstanceGetArgs {
            id: 902,
            scope: QueryScopeArgs::default(),
            realm_kind: None,
        },
    );
    assert!(instance_get.success);
    assert_eq!(instance_get.label.as_deref(), Some("Test Instance"));

    let definition_list = engine_cmd_material_definition_list(
        &mut engine,
        &CmdResourceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![901]),
                ..Default::default()
            },
        },
    );
    assert!(definition_list.success);
    assert_eq!(definition_list.items.len(), 1);
    assert_eq!(definition_list.items[0].id, 901);

    let instance_list = engine_cmd_material_instance_list(
        &mut engine,
        &CmdMaterialInstanceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![902]),
                ..Default::default()
            },
            realm_kind: None,
        },
    );
    assert!(instance_list.success);
    assert_eq!(instance_list.items.len(), 1);
    assert_eq!(instance_list.items[0].id, 902);
}

#[test]
fn material_get_filters_by_realm_kind() {
    let mut engine = test_engine();

    let create_result = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 1001,
            label: Some("mat-3d-only".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: None,
        },
    );
    assert!(create_result.success, "{}", create_result.message);

    let get_ok = engine_cmd_material_get(
        &mut engine,
        &CmdMaterialGetArgs {
            id: 1001,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(get_ok.success, "{}", get_ok.message);

    let get_mismatch = engine_cmd_material_get(
        &mut engine,
        &CmdMaterialGetArgs {
            id: 1001,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(!get_mismatch.success);
    assert_eq!(get_mismatch.message, "Material realm kind mismatch");
}

#[test]
fn material_instance_get_and_list_filter_by_realm_kind() {
    let mut engine = test_engine();

    let definition_result = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 2001,
            slug: "test-def-2001".into(),
            label: Some("Test Definition 2001".into()),
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(definition_result.success, "{}", definition_result.message);

    let instance_result = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 2002,
            slug: "test-def-2001".into(),
            label: Some("Instance 2002".into()),
            options: None,
        },
    );
    assert!(instance_result.success, "{}", instance_result.message);

    {
        let material = engine
            .universal_state
            .scene
            .realm3d
            .materials
            .get_mut(&2002)
            .expect("material instance should create backing material");
        material.realm_kind = MaterialRealmKind::TwoD;
    }

    let get_mismatch = engine_cmd_material_instance_get(
        &mut engine,
        &CmdMaterialInstanceGetArgs {
            id: 2002,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(!get_mismatch.success);
    assert_eq!(
        get_mismatch.message,
        "Material instance realm kind mismatch"
    );

    let get_ok = engine_cmd_material_instance_get(
        &mut engine,
        &CmdMaterialInstanceGetArgs {
            id: 2002,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(get_ok.success, "{}", get_ok.message);

    let list_filtered = engine_cmd_material_instance_list(
        &mut engine,
        &CmdMaterialInstanceListArgs {
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(list_filtered.success);
    assert!(list_filtered.items.iter().all(|entry| entry.id != 2002));

    let list_ok = engine_cmd_material_instance_list(
        &mut engine,
        &CmdMaterialInstanceListArgs {
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(list_ok.success);
    assert!(list_ok.items.iter().any(|entry| entry.id == 2002));
}
