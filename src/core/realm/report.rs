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
    pub target_binds_added: Vec<TargetBindReportKey>,
    pub target_binds_removed: Vec<TargetBindReportKey>,
    pub target_binds_updated: Vec<TargetBindReportKey>,
    pub target_dirty: Vec<u64>,
    pub target_plan_dirty: bool,
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
                .map(|(target, source)| SurfaceCacheEntry {
                    target_surface_id: target.0,
                    source_surface_id: source.0,
                })
                .collect(),
            cache_fallback: cache
                .fallback
                .iter()
                .map(|(target, source)| SurfaceCacheEntry {
                    target_surface_id: target.0,
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
            target_binds_added: Vec::new(),
            target_binds_removed: Vec::new(),
            target_binds_updated: Vec::new(),
            target_dirty: Vec::new(),
            target_plan_dirty: false,
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
        self.target_edges = plan.edges.len();
        if let Some(diff) = diff {
            self.target_added = diff.added_targets.iter().map(|id| id.0).collect();
            self.target_removed = diff.removed_targets.iter().map(|id| id.0).collect();
            self.target_updated = diff.updated_targets.iter().map(|id| id.0).collect();
            self.target_binds_added = diff
                .added_binds
                .iter()
                .map(|(realm_id, target_id)| TargetBindReportKey {
                    realm_id: *realm_id,
                    target_id: target_id.0,
                })
                .collect();
            self.target_binds_removed = diff
                .removed_binds
                .iter()
                .map(|(realm_id, target_id)| TargetBindReportKey {
                    realm_id: *realm_id,
                    target_id: target_id.0,
                })
                .collect();
            self.target_binds_updated = diff
                .updated_binds
                .iter()
                .map(|(realm_id, target_id)| TargetBindReportKey {
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
            self.target_binds_added.clear();
            self.target_binds_removed.clear();
            self.target_binds_updated.clear();
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
    pub target_surface_id: u32,
    pub source_surface_id: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetBindReportKey {
    pub realm_id: u32,
    pub target_id: u64,
}
