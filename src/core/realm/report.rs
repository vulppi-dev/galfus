use serde::{Deserialize, Serialize};

use super::{RealmGraphPlan, SurfaceCache};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameReport {
    pub order: Vec<u32>,
    pub cut_edges: Vec<FrameCutEdge>,
    pub cache_last_good: Vec<SurfaceCacheEntry>,
    pub cache_fallback: Vec<SurfaceCacheEntry>,
    pub blocked_connectors: Vec<u32>,
    pub self_sampled_connectors: Vec<u32>,
    pub throttled_realms: Vec<u32>,
    pub no_progress_realms: Vec<u32>,
    pub target_nodes: usize,
    pub target_edges: usize,
    pub target_added: Vec<u64>,
    pub target_removed: Vec<u64>,
    pub target_updated: Vec<u64>,
    pub target_layers_added: Vec<TargetLayerReportKey>,
    pub target_layers_removed: Vec<TargetLayerReportKey>,
    pub target_layers_updated: Vec<TargetLayerReportKey>,
    pub target_dirty: Vec<u64>,
    pub target_plan_dirty: bool,
    pub target_autolink_failures: Vec<TargetAutoLinkFailure>,
}

impl FrameReport {
    pub fn from_plan(plan: &RealmGraphPlan, cache: &SurfaceCache) -> Self {
        Self {
            order: plan.order.iter().map(|id| id.0).collect(),
            cut_edges: plan
                .cut_edges
                .iter()
                .map(|edge| FrameCutEdge {
                    from: edge.from.0,
                    to: edge.to.0,
                    connector_id: edge.connector_id.map(|id| id.0),
                })
                .collect(),
            cache_last_good: cache
                .last_good
                .iter()
                .map(|(connector, source)| SurfaceCacheEntry {
                    connector_id: connector.0,
                    source_surface_id: source.0,
                })
                .collect(),
            cache_fallback: cache
                .fallback
                .iter()
                .map(|(connector, source)| SurfaceCacheEntry {
                    connector_id: connector.0,
                    source_surface_id: source.0,
                })
                .collect(),
            blocked_connectors: Vec::new(),
            self_sampled_connectors: Vec::new(),
            throttled_realms: Vec::new(),
            no_progress_realms: Vec::new(),
            target_nodes: 0,
            target_edges: 0,
            target_added: Vec::new(),
            target_removed: Vec::new(),
            target_updated: Vec::new(),
            target_layers_added: Vec::new(),
            target_layers_removed: Vec::new(),
            target_layers_updated: Vec::new(),
            target_dirty: Vec::new(),
            target_plan_dirty: false,
            target_autolink_failures: Vec::new(),
        }
    }

    pub fn push_unique(list: &mut Vec<u32>, value: u32) {
        if !list.contains(&value) {
            list.push(value);
        }
    }

    pub fn apply_target_graph_stats(
        &mut self,
        plan: &crate::core::target::TargetGraphPlan,
        diff: Option<&crate::core::target::TargetGraphDiff>,
    ) {
        self.target_nodes = plan.order.len();
        self.target_edges = plan.edges.len().saturating_sub(plan.cut_edges.len());
        if let Some(diff) = diff {
            self.target_added = diff.added_targets.iter().map(|id| id.0).collect();
            self.target_removed = diff.removed_targets.iter().map(|id| id.0).collect();
            self.target_updated = diff.updated_targets.iter().map(|id| id.0).collect();
            self.target_layers_added = diff
                .added_layers
                .iter()
                .map(|(realm_id, target_id)| TargetLayerReportKey {
                    realm_id: *realm_id,
                    target_id: target_id.0,
                })
                .collect();
            self.target_layers_removed = diff
                .removed_layers
                .iter()
                .map(|(realm_id, target_id)| TargetLayerReportKey {
                    realm_id: *realm_id,
                    target_id: target_id.0,
                })
                .collect();
            self.target_layers_updated = diff
                .updated_layers
                .iter()
                .map(|(realm_id, target_id)| TargetLayerReportKey {
                    realm_id: *realm_id,
                    target_id: target_id.0,
                })
                .collect();
            self.target_dirty = diff.dirty_targets.iter().map(|id| id.0).collect();
            self.target_plan_dirty = diff.plan_dirty;
        } else {
            self.target_added.clear();
            self.target_removed.clear();
            self.target_updated.clear();
            self.target_layers_added.clear();
            self.target_layers_removed.clear();
            self.target_layers_updated.clear();
            self.target_dirty.clear();
            self.target_plan_dirty = false;
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameCutEdge {
    pub from: u32,
    pub to: u32,
    pub connector_id: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceCacheEntry {
    pub connector_id: u32,
    pub source_surface_id: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetLayerReportKey {
    pub realm_id: u32,
    pub target_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TargetAutoLinkFailure {
    pub realm_id: u32,
    pub target_id: u64,
    pub reason: String,
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

        report.apply_target_graph_stats(&plan, Some(&diff));
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
