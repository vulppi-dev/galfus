use super::*;
use crate::core::realm::RealmId;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdMaterialDefinitionCreateArgs,
    CmdMaterialInstanceCreateArgs, EnvironmentConfig, ShaderMaterialPreset,
    engine_cmd_camera_create, engine_cmd_environment_create, engine_cmd_material_definition_create,
    engine_cmd_material_instance_create,
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
            preset: ShaderMaterialPreset::Standard,
            shader_type: None,
            shader_source: "fn vertex(input: VertexInput) -> VertexOutput { var out: VertexOutput; out.world_position = input.position; out.world_normal = input.normal; out.uv = input.uv; out.clip_position = vec4<f32>(0.0); return out; } fn fragment(input: FragmentInput) -> FragmentOutput { var out: FragmentOutput; out.color = vec4<f32>(1.0); out.emissive = vec4<f32>(0.0); return out; }".into(),
            shader_params_schema: None,
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
        &CmdResourceGetArgs {
            id: 902,
            scope: QueryScopeArgs::default(),
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
        &CmdResourceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![902]),
                ..Default::default()
            },
        },
    );
    assert!(instance_list.success);
    assert_eq!(instance_list.items.len(), 1);
    assert_eq!(instance_list.items[0].id, 902);
}
