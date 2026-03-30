pub use vulfram_scene_core::{
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
mod tests {
    use super::*;
    use crate::core::realm::{
        ConnectorId, RealmGraphEdge, RealmGraphPlan, SurfaceCache, SurfaceId,
    };
    use crate::core::target::{TargetGraphDiff, TargetGraphPlan, TargetId};

    #[test]
    fn push_unique_only_inserts_once() {
        let mut list = vec![1, 2];
        FrameReport::push_unique(&mut list, 2);
        FrameReport::push_unique(&mut list, 3);
        assert_eq!(list, vec![1, 2, 3]);
    }

    #[test]
    fn apply_target_graph_stats_copies_plan_and_diff_data() {
        let mut report = FrameReport::default();
        let plan = TargetGraphPlan {
            edges: vec![
                crate::core::target::TargetEdge {
                    parent: TargetId(1),
                    child: TargetId(2),
                },
                crate::core::target::TargetEdge {
                    parent: TargetId(2),
                    child: TargetId(3),
                },
            ],
            order: vec![TargetId(1), TargetId(2), TargetId(3)],
            cut_edges: vec![crate::core::target::TargetEdge {
                parent: TargetId(2),
                child: TargetId(3),
            }],
        };
        let diff = TargetGraphDiff {
            added_targets: vec![TargetId(10)],
            removed_targets: vec![TargetId(11)],
            updated_targets: vec![TargetId(12)],
            added_layers: vec![(7, TargetId(20))],
            removed_layers: vec![(8, TargetId(21))],
            updated_layers: vec![(9, TargetId(22))],
            dirty_targets: vec![TargetId(12)],
            plan_dirty: true,
        };

        apply_target_graph_stats(&mut report, &plan, Some(&diff));
        assert_eq!(report.target_nodes, 3);
        assert_eq!(report.target_edges, 1);
        assert_eq!(report.target_added, vec![10]);
        assert_eq!(report.target_plan_dirty, true);
    }

    #[test]
    fn from_plan_serializes_realm_order_edges_and_cache() {
        let plan = RealmGraphPlan {
            order: vec![
                crate::core::realm::RealmId(3),
                crate::core::realm::RealmId(4),
            ],
            cut_edges: vec![RealmGraphEdge {
                from: crate::core::realm::RealmId(3),
                to: crate::core::realm::RealmId(4),
                connector_id: Some(ConnectorId(9)),
            }],
        };
        let mut cache = SurfaceCache::default();
        cache.last_good.insert(ConnectorId(2), SurfaceId(5));
        cache.fallback.insert(ConnectorId(3), SurfaceId(6));

        let report = FrameReport::from_plan(&plan, &cache);
        assert_eq!(report.order, vec![3, 4]);
        assert_eq!(report.cut_edges.len(), 1);
        assert_eq!(report.cache_last_good.len(), 1);
        assert_eq!(report.cache_fallback.len(), 1);
    }
}
