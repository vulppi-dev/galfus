use crate::core::realm::{CmdRealmCreateArgs, RealmId, RealmKindDto, engine_cmd_realm_create};
use crate::core::state::EngineState;
use crate::core::ui::cmd::{CmdUiDocumentCreateArgs, engine_cmd_ui_document_create};
use crate::core::ui::input::process_ui_input;

#[test]
fn ui_input_converts_pointer_button_into_egui_pointer_event() {
    let mut engine = EngineState::new();
    let created_realm = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    );
    let realm_id = created_realm.realm_id.expect("realm id");
    let _ = engine_cmd_ui_document_create(
        &mut engine,
        &CmdUiDocumentCreateArgs {
            document_id: 900,
            realm_id,
            rect: glam::vec4(0.0, 0.0, 320.0, 240.0),
            theme_id: None,
        },
    );
    let surface_id =
        engine
            .universal_state
            .composition
            .surfaces
            .alloc(crate::core::realm::SurfaceState {
                kind: crate::core::realm::SurfaceKind::Offscreen,
                size: glam::uvec2(320, 240),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            });
    if let Some(realm_entry) = engine
        .universal_state
        .composition
        .realms
        .entries
        .get_mut(&RealmId(realm_id))
    {
        realm_entry.value.output_surface = Some(surface_id);
    }
    engine
        .runtime
        .push_event(crate::core::cmd::EngineEvent::Pointer(
            crate::core::input::events::PointerEvent::OnButton {
                window_id: 1,
                window_width: Some(320),
                window_height: Some(240),
                pointer_type: 0,
                pointer_id: 1,
                button: 1,
                state: crate::core::input::events::ElementState::Pressed,
                position: glam::vec2(100.0, 80.0),
                position_target: None,
                target_width: Some(320),
                target_height: Some(240),
                trace: Some(crate::core::input::events::PointerEventTrace {
                    window_id: 1,
                    realm_id,
                    target_id: None,
                    connector_id: None,
                    source_realm_id: None,
                    uv: None,
                    hops: Vec::new(),
                }),
            },
        ));

    process_ui_input(&mut engine);

    let realm_state = engine
        .universal_state
        .interaction
        .ui
        .realms
        .get(&RealmId(realm_id))
        .expect("ui realm");
    assert!(
        realm_state
            .pending_events
            .iter()
            .any(|event| matches!(event, egui::Event::PointerButton { pressed: true, .. }))
    );
}
