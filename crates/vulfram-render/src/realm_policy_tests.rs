use super::{clear_alpha_for_realm_kind, graph_is_compatible_with_realm_kind};
use crate::{LogicalId, RenderGraphNode, RenderGraphPlan};
use vulfram_realm_core::{RENDER_PASS_FORWARD, RENDER_PASS_UI, RealmKind};

#[test]
fn realm_policy_maps_clear_alpha_by_kind() {
    assert_eq!(clear_alpha_for_realm_kind(RealmKind::ThreeD), 1.0);
    assert_eq!(clear_alpha_for_realm_kind(RealmKind::TwoD), 0.0);
}

#[test]
fn realm_policy_validates_passes_by_kind() {
    let ui_plan = RenderGraphPlan {
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(RENDER_PASS_UI.into()),
            pass_id: RENDER_PASS_UI.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            require: Vec::new(),
            priority: 0,
            enabled: true,
            params: Default::default(),
        }],
        order: vec![0],
    };
    let forward_plan = RenderGraphPlan {
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(RENDER_PASS_FORWARD.into()),
            pass_id: RENDER_PASS_FORWARD.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            require: Vec::new(),
            priority: 0,
            enabled: true,
            params: Default::default(),
        }],
        order: vec![0],
    };

    assert!(graph_is_compatible_with_realm_kind(
        &ui_plan,
        RealmKind::TwoD
    ));
    assert!(!graph_is_compatible_with_realm_kind(
        &forward_plan,
        RealmKind::TwoD
    ));
    assert!(graph_is_compatible_with_realm_kind(
        &forward_plan,
        RealmKind::ThreeD
    ));
}
