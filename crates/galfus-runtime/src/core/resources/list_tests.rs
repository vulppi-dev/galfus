use super::*;
use crate::core::resources::{
    CmdMaterialCreateArgs, MaterialKind, MaterialRealmKind, engine_cmd_material_create,
};
use crate::core::test_support::test_engine;

#[test]
fn material_list_without_filter_returns_all_materials() {
    let mut engine = test_engine();

    let create_3d = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 3101,
            label: Some("mat-3d".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: None,
        },
    );
    assert!(create_3d.success, "{}", create_3d.message);

    let create_2d = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 3102,
            label: Some("mat-2d".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: None,
        },
    );
    assert!(create_2d.success, "{}", create_2d.message);

    let listed = engine_cmd_material_list(
        &mut engine,
        &CmdMaterialListArgs {
            window_id: 0,
            realm_kind: None,
        },
    );
    assert!(listed.success);
    assert!(listed.materials.iter().any(|entry| entry.id == 3101));
    assert!(listed.materials.iter().any(|entry| entry.id == 3102));
}

#[test]
fn material_list_with_realm_kind_filter_returns_compatible_materials() {
    let mut engine = test_engine();

    let create_3d = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 3201,
            label: Some("mat-3d-only".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: None,
        },
    );
    assert!(create_3d.success, "{}", create_3d.message);

    let create_2d = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 3202,
            label: Some("mat-2d-only".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: None,
        },
    );
    assert!(create_2d.success, "{}", create_2d.message);

    let list_2d = engine_cmd_material_list(
        &mut engine,
        &CmdMaterialListArgs {
            window_id: 0,
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(list_2d.success);
    assert!(!list_2d.materials.iter().any(|entry| entry.id == 3201));
    assert!(list_2d.materials.iter().any(|entry| entry.id == 3202));

    let list_3d = engine_cmd_material_list(
        &mut engine,
        &CmdMaterialListArgs {
            window_id: 0,
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(list_3d.success);
    assert!(list_3d.materials.iter().any(|entry| entry.id == 3201));
    assert!(!list_3d.materials.iter().any(|entry| entry.id == 3202));
}
