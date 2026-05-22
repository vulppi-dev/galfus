use super::*;
use crate::core::test_support::test_engine;

#[test]
fn realm_get_returns_created_realm() {
    let mut engine = test_engine();
    let created = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::ThreeD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    );
    assert!(created.success);
    let realm_id = created.realm_id.expect("realm id should exist");

    let get = engine_cmd_realm_get(&mut engine, &CmdRealmGetArgs { realm_id });
    assert!(get.success);
    assert_eq!(get.realm_id, realm_id);
    assert_eq!(get.kind, Some(RealmKindDto::ThreeD));
    assert!(get.render_graph_id.is_some());
}

#[test]
fn realm_get_returns_not_found_for_unknown_id() {
    let mut engine = test_engine();
    let get = engine_cmd_realm_get(&mut engine, &CmdRealmGetArgs { realm_id: 999_999 });
    assert!(!get.success);
    assert!(get.message.contains("not found"));
}

#[test]
fn realm_list_filters_by_kind_and_ids() {
    let mut engine = test_engine();
    let realm_3d = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::ThreeD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    )
    .realm_id
    .expect("3d realm id should exist");
    let realm_2d = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    )
    .realm_id
    .expect("2d realm id should exist");

    let by_kind = engine_cmd_realm_list(
        &mut engine,
        &CmdRealmListArgs {
            kind: Some(RealmKindDto::ThreeD),
            ids: None,
        },
    );
    assert!(by_kind.success);
    assert!(
        by_kind
            .items
            .iter()
            .all(|item| item.kind == RealmKindDto::ThreeD)
    );

    let by_id = engine_cmd_realm_list(
        &mut engine,
        &CmdRealmListArgs {
            kind: None,
            ids: Some(vec![realm_2d]),
        },
    );
    assert!(by_id.success);
    assert_eq!(by_id.items.len(), 1);
    assert_eq!(by_id.items[0].realm_id, realm_2d);
    assert_ne!(realm_3d, realm_2d);

    let realm_3d_id = crate::core::realm::RealmId(realm_3d);
    let realm_2d_id = crate::core::realm::RealmId(realm_2d);

    assert!(
        engine
            .universal_state
            .scene
            .realm3d
            .entities
            .contains_key(&realm_3d_id)
    );
    assert!(
        !engine
            .universal_state
            .scene
            .realm2d
            .entities
            .contains_key(&realm_3d_id)
    );
    assert!(
        engine
            .universal_state
            .scene
            .realm2d
            .entities
            .contains_key(&realm_2d_id)
    );
    assert!(
        !engine
            .universal_state
            .scene
            .realm3d
            .entities
            .contains_key(&realm_2d_id)
    );
}
