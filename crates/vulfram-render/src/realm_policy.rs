use crate::RenderGraphPlan;
use vulfram_realm_core::RealmKind;

pub fn graph_is_compatible_with_realm_kind(plan: &RenderGraphPlan, realm_kind: RealmKind) -> bool {
    let pass_ids = plan.nodes.iter().map(|node| node.pass_id.as_str());
    match realm_kind {
        RealmKind::ThreeD => vulfram_realm_3d::graph_is_compatible(pass_ids),
        RealmKind::TwoD => vulfram_realm_2d::graph_is_compatible(pass_ids),
    }
}

pub fn clear_alpha_for_realm_kind(realm_kind: RealmKind) -> f64 {
    match realm_kind {
        RealmKind::ThreeD => 1.0,
        RealmKind::TwoD => 0.0,
    }
}

pub fn supports_render_pass_for_realm_kind(pass_id: &str, realm_kind: RealmKind) -> bool {
    match realm_kind {
        RealmKind::ThreeD => vulfram_realm_3d::supports_render_pass(pass_id),
        RealmKind::TwoD => vulfram_realm_2d::supports_render_pass(pass_id),
    }
}

#[cfg(test)]
mod tests {
    use super::{clear_alpha_for_realm_kind, graph_is_compatible_with_realm_kind};
    use crate::{LogicalId, RenderGraphNode, RenderGraphPlan};
    use vulfram_realm_core::RealmKind;

    #[test]
    fn realm_policy_maps_clear_alpha_by_kind() {
        assert_eq!(clear_alpha_for_realm_kind(RealmKind::ThreeD), 1.0);
        assert_eq!(clear_alpha_for_realm_kind(RealmKind::TwoD), 0.0);
    }

    #[test]
    fn realm_policy_validates_passes_by_kind() {
        let ui_plan = RenderGraphPlan {
            nodes: vec![RenderGraphNode {
                node_id: LogicalId::Str("ui".into()),
                pass_id: "ui".into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                params: Default::default(),
            }],
            order: vec![0],
        };
        let forward_plan = RenderGraphPlan {
            nodes: vec![RenderGraphNode {
                node_id: LogicalId::Str("forward".into()),
                pass_id: "forward".into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
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
}
