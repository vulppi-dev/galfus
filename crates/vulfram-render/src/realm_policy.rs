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
#[path = "realm_policy_tests.rs"]
mod tests;
