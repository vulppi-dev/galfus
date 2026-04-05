pub use vulfram_realm_core::{
    FrameCutEdge, FrameReport, SurfaceCacheEntry, TargetAutoLinkFailure, TargetLayerReportKey,
};

pub fn apply_target_graph_stats(
    report: &mut FrameReport,
    plan: &crate::core::target::TargetGraphPlan,
    diff: Option<&crate::core::target::TargetGraphDiff>,
) {
    report.target_nodes = plan.order.len();
    report.target_edges = plan.edges.len().saturating_sub(plan.cut_edges.len());
    if let Some(diff) = diff {
        report.target_added = diff.added_targets.iter().map(|id| id.0).collect();
        report.target_removed = diff.removed_targets.iter().map(|id| id.0).collect();
        report.target_updated = diff.updated_targets.iter().map(|id| id.0).collect();
        report.target_layers_added = diff
            .added_layers
            .iter()
            .map(|(realm_id, target_id)| TargetLayerReportKey {
                realm_id: *realm_id,
                target_id: target_id.0,
            })
            .collect();
        report.target_layers_removed = diff
            .removed_layers
            .iter()
            .map(|(realm_id, target_id)| TargetLayerReportKey {
                realm_id: *realm_id,
                target_id: target_id.0,
            })
            .collect();
        report.target_layers_updated = diff
            .updated_layers
            .iter()
            .map(|(realm_id, target_id)| TargetLayerReportKey {
                realm_id: *realm_id,
                target_id: target_id.0,
            })
            .collect();
        report.target_dirty = diff.dirty_targets.iter().map(|id| id.0).collect();
        report.target_plan_dirty = diff.plan_dirty;
    } else {
        report.target_added.clear();
        report.target_removed.clear();
        report.target_updated.clear();
        report.target_layers_added.clear();
        report.target_layers_removed.clear();
        report.target_layers_updated.clear();
        report.target_dirty.clear();
        report.target_plan_dirty = false;
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
