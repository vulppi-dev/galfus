use super::{
    DEFAULT_2D_RENDER_GRAPH_ID, DEFAULT_3D_RENDER_GRAPH_ID, fallback_render_graph_id,
    is_reserved_render_graph_id, resolve_render_graph_id,
};

#[test]
fn render_graph_id_helpers_follow_realm_kind_defaults() {
    assert_eq!(
        fallback_render_graph_id(galfus_realm_core::RealmKind::ThreeD),
        DEFAULT_3D_RENDER_GRAPH_ID
    );
    assert_eq!(
        fallback_render_graph_id(galfus_realm_core::RealmKind::TwoD),
        DEFAULT_2D_RENDER_GRAPH_ID
    );
    assert_eq!(
        resolve_render_graph_id(None, galfus_realm_core::RealmKind::ThreeD),
        DEFAULT_3D_RENDER_GRAPH_ID
    );
    assert_eq!(
        resolve_render_graph_id(Some(77), galfus_realm_core::RealmKind::TwoD),
        77
    );
    assert!(is_reserved_render_graph_id(DEFAULT_3D_RENDER_GRAPH_ID));
    assert!(is_reserved_render_graph_id(DEFAULT_2D_RENDER_GRAPH_ID));
    assert!(!is_reserved_render_graph_id(77));
}
