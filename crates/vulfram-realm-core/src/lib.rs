mod render_passes;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};

pub use render_passes::{
    RENDER_PASS_BLOOM, RENDER_PASS_COMPOSE, RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL,
    RENDER_PASS_OUTLINE, RENDER_PASS_POST, RENDER_PASS_SHADOW, RENDER_PASS_SKYBOX,
    RENDER_PASS_SSAO, RENDER_PASS_SSAO_BLUR, RENDER_PASS_UI,
};
pub use vulfram_types::{ConnectorId, PresentId, RealmId, RealmKind, SurfaceId};

#[derive(Debug, Clone)]
pub struct RealmState {
    pub kind: RealmKind,
    pub output_surface: Option<SurfaceId>,
    pub render_graph_id: Option<u32>,
    pub importance: u8,
    pub cache_policy: u8,
    pub last_render_frame: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectorState {
    pub target_realm: RealmId,
    pub source_surface: SurfaceId,
    pub rect: glam::Vec4,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<glam::Vec4>,
    pub input_flags: u32,
}

#[derive(Debug, Clone)]
pub struct PresentState {
    pub window_id: u32,
    pub surface: SurfaceId,
}

#[derive(Debug, Clone)]
pub struct AutoLink {
    pub surface_id: SurfaceId,
    pub connector_id: Option<ConnectorId>,
    pub present_id: Option<PresentId>,
}

#[derive(Debug, Default)]
pub struct SurfaceCache {
    pub last_good: HashMap<ConnectorId, SurfaceId>,
    pub fallback: HashMap<ConnectorId, SurfaceId>,
}

#[derive(Debug, Clone)]
pub struct RealmGraphEdge {
    pub from: RealmId,
    pub to: RealmId,
    pub connector_id: Option<ConnectorId>,
}

#[derive(Debug, Default)]
pub struct RealmGraphPlan {
    pub order: Vec<RealmId>,
    pub cut_edges: Vec<RealmGraphEdge>,
}

#[derive(Debug, Default)]
pub struct RealmGraphPlanner;

impl RealmGraphPlanner {
    pub fn build_plan(
        &self,
        realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
        presents: &[(u32, SurfaceId)],
        connectors: &[(ConnectorId, SurfaceId, RealmId)],
    ) -> RealmGraphPlan {
        let mut edges = Vec::new();
        let mut hard_targets = HashSet::new();
        let surface_to_realm = collect_surface_to_realm(realm_output_surfaces);

        for (_, surface_id) in presents {
            if let Some(realm_id) = surface_to_realm.get(surface_id).copied() {
                hard_targets.insert(realm_id);
            }
        }

        for (connector_id, source_surface, target_realm) in connectors {
            if let Some(source_realm) = surface_to_realm.get(source_surface).copied() {
                edges.push(RealmGraphEdge {
                    from: source_realm,
                    to: *target_realm,
                    connector_id: Some(*connector_id),
                });
            }
        }

        let mut all_realms: HashSet<RealmId> = realm_output_surfaces.keys().copied().collect();
        all_realms.extend(hard_targets);
        let (order, cut_edges) = topo_with_soft_cuts(&all_realms, &edges);

        RealmGraphPlan { order, cut_edges }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Window,
    WidgetRealmViewport,
    RealmPlane,
    Texture,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(tag = "unit", content = "value", rename_all = "kebab-case")]
pub enum DimensionValue {
    Px(f32),
    Percent(f32),
    Character(f32),
    Display(f32),
}

impl DimensionValue {
    pub fn resolve(self, reference: f32, char_width: f32) -> f32 {
        match self {
            Self::Px(value) => value,
            Self::Percent(value) => (value / 100.0) * reference,
            Self::Character(value) => value * char_width,
            Self::Display(value) => value * 4.0,
        }
    }
}

impl Default for DimensionValue {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetLayerLayout {
    pub left: DimensionValue,
    pub top: DimensionValue,
    pub width: DimensionValue,
    pub height: DimensionValue,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<glam::Vec4>,
}

impl Default for TargetLayerLayout {
    fn default() -> Self {
        Self {
            left: DimensionValue::Px(0.0),
            top: DimensionValue::Px(0.0),
            width: DimensionValue::Percent(100.0),
            height: DimensionValue::Percent(100.0),
            z_index: 0,
            blend_mode: 0,
            clip: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetLayerState {
    pub realm_id: u32,
    pub target_id: TargetId,
    pub layout: TargetLayerLayout,
    pub camera_id: Option<u32>,
    pub environment_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetEdge {
    pub parent: TargetId,
    pub child: TargetId,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphPlan {
    pub edges: Vec<TargetEdge>,
    pub order: Vec<TargetId>,
    pub cut_edges: Vec<TargetEdge>,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphDiff {
    pub added_targets: Vec<TargetId>,
    pub removed_targets: Vec<TargetId>,
    pub updated_targets: Vec<TargetId>,
    pub added_layers: Vec<(u32, TargetId)>,
    pub removed_layers: Vec<(u32, TargetId)>,
    pub updated_layers: Vec<(u32, TargetId)>,
    pub dirty_targets: Vec<TargetId>,
    pub plan_dirty: bool,
}

#[derive(Debug, Default)]
pub struct TargetGraphPlanner;

impl TargetGraphPlanner {
    pub fn build_plan(
        &self,
        targets: &HashMap<TargetId, (TargetKind, Option<u32>)>,
        layers: &HashMap<(u32, TargetId), TargetLayerState>,
        realms: &HashSet<RealmId>,
    ) -> TargetGraphPlan {
        let window_targets = collect_window_targets(targets);
        let layers_by_target = collect_layers_by_target(layers);
        let realm_windows = collect_realm_windows(targets, layers, realms);
        let mut edges = Vec::with_capacity(targets.len());

        for (target_id, (kind, _window_id)) in targets {
            match kind {
                TargetKind::Window | TargetKind::Texture => {}
                TargetKind::WidgetRealmViewport | TargetKind::RealmPlane => {
                    if let Some(parent) = infer_parent_from_layers(
                        &layers_by_target,
                        &realm_windows,
                        *target_id,
                        &window_targets,
                    ) {
                        edges.push(TargetEdge {
                            parent,
                            child: *target_id,
                        });
                    }
                }
            }
        }

        edges.sort_by_key(|edge| (edge.parent.0, edge.child.0));
        let all_targets: HashSet<TargetId> = targets.keys().copied().collect();
        let (order, cut_edges) = topo_targets_with_soft_cuts(&all_targets, &edges);

        TargetGraphPlan {
            edges,
            order,
            cut_edges,
        }
    }
}

fn collect_window_targets(
    targets: &HashMap<TargetId, (TargetKind, Option<u32>)>,
) -> HashMap<u32, TargetId> {
    let mut map: HashMap<u32, TargetId> = HashMap::new();
    for (target_id, (kind, window_id)) in targets {
        if *kind != TargetKind::Window {
            continue;
        }
        if let Some(window_id) = *window_id {
            if let Some(existing) = map.get_mut(&window_id) {
                if target_id.0 < existing.0 {
                    *existing = *target_id;
                }
            } else {
                map.insert(window_id, *target_id);
            }
        }
    }
    map
}

fn collect_layers_by_target(
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
) -> HashMap<TargetId, Vec<u32>> {
    let mut by_target = HashMap::new();
    for ((realm_id, target_id), _) in layers {
        by_target
            .entry(*target_id)
            .or_insert_with(Vec::new)
            .push(*realm_id);
    }
    by_target
}

fn collect_realm_windows(
    targets: &HashMap<TargetId, (TargetKind, Option<u32>)>,
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    realms: &HashSet<RealmId>,
) -> HashMap<u32, u32> {
    let mut map = HashMap::new();
    for ((realm_id, target_id), _) in layers {
        if !realms.contains(&RealmId(*realm_id)) {
            continue;
        }
        let Some((kind, window_id)) = targets.get(target_id) else {
            continue;
        };
        if *kind != TargetKind::Window {
            continue;
        }
        let Some(window_id) = *window_id else {
            continue;
        };
        match map.get_mut(realm_id) {
            Some(existing_window_id) => {
                if window_id < *existing_window_id {
                    *existing_window_id = window_id;
                }
            }
            None => {
                map.insert(*realm_id, window_id);
            }
        }
    }
    map
}

fn infer_parent_from_layers(
    layers_by_target: &HashMap<TargetId, Vec<u32>>,
    realm_windows: &HashMap<u32, u32>,
    target_id: TargetId,
    window_targets: &HashMap<u32, TargetId>,
) -> Option<TargetId> {
    let mut chosen_window = None;
    let mut chosen_realm = None;

    let realm_ids = layers_by_target.get(&target_id)?;
    for layer_realm_id in realm_ids {
        let Some(realm_window_id) = realm_windows.get(layer_realm_id).copied() else {
            continue;
        };

        match chosen_window {
            None => {
                chosen_window = Some(realm_window_id);
                chosen_realm = Some(*layer_realm_id);
            }
            Some(current_window) => {
                if realm_window_id < current_window {
                    chosen_window = Some(realm_window_id);
                    chosen_realm = Some(*layer_realm_id);
                } else if realm_window_id == current_window {
                    let current_realm = chosen_realm.unwrap_or(u32::MAX);
                    if *layer_realm_id < current_realm {
                        chosen_realm = Some(*layer_realm_id);
                    }
                }
            }
        }
    }

    let window_id = chosen_window?;
    window_targets.get(&window_id).copied()
}

fn topo_targets_with_soft_cuts(
    targets: &HashSet<TargetId>,
    edges: &[TargetEdge],
) -> (Vec<TargetId>, Vec<TargetEdge>) {
    let mut final_order = Vec::new();
    let mut cut_edges = Vec::new();
    let mut remaining_targets: HashSet<TargetId> = targets.iter().copied().collect();
    let mut remaining_edges: Vec<TargetEdge> = edges.to_vec();
    let mut guard = 0;

    while !remaining_targets.is_empty() {
        guard += 1;
        if guard > 64 {
            let mut leftover: Vec<_> = remaining_targets.iter().copied().collect();
            leftover.sort_by_key(|id| id.0);
            final_order.extend(leftover);
            break;
        }

        let order = topo_target_order(&remaining_targets, &remaining_edges);
        for node in &order {
            remaining_targets.remove(node);
        }
        final_order.extend(order);

        if remaining_targets.is_empty() {
            break;
        }

        let mut pruned = Vec::new();
        for edge in remaining_edges {
            if remaining_targets.contains(&edge.parent) && remaining_targets.contains(&edge.child) {
                cut_edges.push(edge);
            } else {
                pruned.push(edge);
            }
        }
        remaining_edges = pruned;
    }

    (final_order, cut_edges)
}

fn topo_target_order(targets: &HashSet<TargetId>, edges: &[TargetEdge]) -> Vec<TargetId> {
    let mut incoming: HashMap<TargetId, usize> = targets.iter().map(|id| (*id, 0)).collect();

    for edge in edges {
        if incoming.contains_key(&edge.child) {
            *incoming.entry(edge.child).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<TargetId> = incoming
        .iter()
        .filter_map(|(id, count)| if *count == 0 { Some(*id) } else { None })
        .collect();
    let mut queue_vec: Vec<_> = queue.drain(..).collect();
    queue_vec.sort_by_key(|id| id.0);
    let mut queue: VecDeque<TargetId> = queue_vec.into();

    let mut edges_by_parent: HashMap<TargetId, Vec<TargetId>> = HashMap::new();
    for edge in edges {
        edges_by_parent
            .entry(edge.parent)
            .or_default()
            .push(edge.child);
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(children) = edges_by_parent.get(&node) {
            for child in children {
                if let Some(count) = incoming.get_mut(child) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        queue.push_back(*child);
                    }
                }
            }
        }
    }

    order
}

fn collect_surface_to_realm(
    realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
) -> HashMap<SurfaceId, RealmId> {
    let mut map = HashMap::new();
    for (realm_id, surface_id) in realm_output_surfaces {
        if let Some(surface_id) = surface_id {
            map.insert(*surface_id, *realm_id);
        }
    }
    map
}

fn topo_with_soft_cuts(
    realms: &HashSet<RealmId>,
    edges: &[RealmGraphEdge],
) -> (Vec<RealmId>, Vec<RealmGraphEdge>) {
    let mut final_order = Vec::new();
    let mut cut_edges = Vec::new();
    let mut remaining_realms: HashSet<RealmId> = realms.iter().copied().collect();
    let mut remaining_edges: Vec<RealmGraphEdge> = edges.to_vec();
    let mut guard = 0;

    while !remaining_realms.is_empty() {
        guard += 1;
        if guard > 32 {
            let mut leftover: Vec<_> = remaining_realms.iter().copied().collect();
            leftover.sort_by_key(|id| id.0);
            final_order.extend(leftover);
            break;
        }

        let order = topo_order(&remaining_realms, &remaining_edges);
        for node in &order {
            remaining_realms.remove(node);
        }
        final_order.extend(order.iter().copied());

        if remaining_realms.is_empty() {
            break;
        }

        let mut pruned = Vec::new();
        for edge in remaining_edges {
            if remaining_realms.contains(&edge.from) && remaining_realms.contains(&edge.to) {
                cut_edges.push(edge);
            } else {
                pruned.push(edge);
            }
        }
        remaining_edges = pruned;
    }

    (final_order, cut_edges)
}

fn topo_order(realms: &HashSet<RealmId>, edges: &[RealmGraphEdge]) -> Vec<RealmId> {
    let mut incoming: HashMap<RealmId, usize> = realms.iter().map(|id| (*id, 0)).collect();
    let mut edges_by_from: HashMap<RealmId, Vec<RealmId>> = HashMap::new();
    for edge in edges {
        if realms.contains(&edge.to) {
            *incoming.entry(edge.to).or_insert(0) += 1;
        }
        if realms.contains(&edge.from) && realms.contains(&edge.to) {
            edges_by_from.entry(edge.from).or_default().push(edge.to);
        }
    }

    let mut queue: VecDeque<RealmId> = incoming
        .iter()
        .filter_map(|(id, count)| if *count == 0 { Some(*id) } else { None })
        .collect();
    let mut queue_vec: Vec<_> = queue.drain(..).collect();
    queue_vec.sort_by_key(|id| id.0);
    let mut queue: VecDeque<RealmId> = queue_vec.into();

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(children) = edges_by_from.get(&node) {
            for child in children {
                if let Some(entry) = incoming.get_mut(child) {
                    *entry = entry.saturating_sub(1);
                    if *entry == 0 {
                        queue.push_back(*child);
                    }
                }
            }
        }
    }

    order
}

#[derive(Debug, Clone)]
pub struct TableEntry<T> {
    pub value: T,
}

impl<T> TableEntry<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

#[derive(Debug, Default)]
pub struct RealmTable {
    pub next_id: u32,
    pub entries: HashMap<RealmId, TableEntry<RealmState>>,
}

impl RealmTable {
    pub fn alloc(&mut self, state: RealmState) -> RealmId {
        let id = RealmId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.insert(id, TableEntry::new(state));
        id
    }

    pub fn get(&self, id: RealmId) -> Option<&TableEntry<RealmState>> {
        self.entries.get(&id)
    }

    pub fn remove(&mut self, id: RealmId) -> Option<TableEntry<RealmState>> {
        self.entries.remove(&id)
    }
}

#[derive(Debug, Default)]
pub struct ConnectorTable {
    pub next_id: u32,
    pub entries: HashMap<ConnectorId, TableEntry<ConnectorState>>,
}

impl ConnectorTable {
    pub fn alloc(&mut self, state: ConnectorState) -> ConnectorId {
        let id = ConnectorId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.insert(id, TableEntry::new(state));
        id
    }

    pub fn get_mut(&mut self, id: ConnectorId) -> Option<&mut TableEntry<ConnectorState>> {
        self.entries.get_mut(&id)
    }

    pub fn remove(&mut self, id: ConnectorId) -> Option<TableEntry<ConnectorState>> {
        self.entries.remove(&id)
    }
}

#[derive(Debug, Default)]
pub struct PresentTable {
    pub next_id: u32,
    pub entries: HashMap<PresentId, TableEntry<PresentState>>,
}

impl PresentTable {
    pub fn alloc(&mut self, state: PresentState) -> PresentId {
        let id = PresentId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.insert(id, TableEntry::new(state));
        id
    }

    pub fn remove(&mut self, id: PresentId) -> Option<TableEntry<PresentState>> {
        self.entries.remove(&id)
    }

    pub fn remove_by_window(&mut self, window_id: u32) {
        self.entries
            .retain(|_, entry| entry.value.window_id != window_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_table_allocates_monotonic_ids() {
        let mut table = RealmTable::default();
        let first = table.alloc(RealmState {
            kind: RealmKind::ThreeD,
            output_surface: None,
            render_graph_id: None,
            importance: 1,
            cache_policy: 0,
            last_render_frame: 0,
        });
        let second = table.alloc(RealmState {
            kind: RealmKind::TwoD,
            output_surface: Some(SurfaceId(3)),
            render_graph_id: Some(9),
            importance: 2,
            cache_policy: 1,
            last_render_frame: 7,
        });

        assert_eq!(first, RealmId(0));
        assert_eq!(second, RealmId(1));
    }

    #[test]
    fn present_table_remove_by_window_prunes_matching_entries() {
        let mut table = PresentTable::default();
        let keep = table.alloc(PresentState {
            window_id: 1,
            surface: SurfaceId(10),
        });
        let _drop = table.alloc(PresentState {
            window_id: 2,
            surface: SurfaceId(20),
        });

        table.remove_by_window(2);

        assert!(table.entries.contains_key(&keep));
        assert_eq!(table.entries.len(), 1);
    }

    #[test]
    fn planner_orders_linear_dependency() {
        let realms = HashMap::from([
            (RealmId(0), Some(SurfaceId(10))),
            (RealmId(1), Some(SurfaceId(11))),
        ]);
        let presents = vec![(1, SurfaceId(11))];
        let connectors = vec![(ConnectorId(2), SurfaceId(10), RealmId(1))];

        let plan = RealmGraphPlanner.build_plan(&realms, &presents, &connectors);
        assert_eq!(plan.order, vec![RealmId(0), RealmId(1)]);
        assert!(plan.cut_edges.is_empty());
    }

    #[test]
    fn planner_cuts_cycles_deterministically() {
        let realms = HashMap::from([
            (RealmId(0), Some(SurfaceId(10))),
            (RealmId(1), Some(SurfaceId(11))),
        ]);
        let presents = Vec::new();
        let connectors = vec![
            (ConnectorId(2), SurfaceId(10), RealmId(1)),
            (ConnectorId(3), SurfaceId(11), RealmId(0)),
        ];

        let plan = RealmGraphPlanner.build_plan(&realms, &presents, &connectors);
        assert_eq!(plan.order, vec![RealmId(0), RealmId(1)]);
        assert_eq!(plan.cut_edges.len(), 2);
    }

    #[test]
    fn frame_report_serializes_realm_order_edges_and_cache() {
        let plan = RealmGraphPlan {
            order: vec![RealmId(3), RealmId(4)],
            cut_edges: vec![RealmGraphEdge {
                from: RealmId(3),
                to: RealmId(4),
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

    #[test]
    fn dimension_value_percent_uses_reference_axis() {
        let value = DimensionValue::Percent(25.0);
        assert_eq!(value.resolve(400.0, 8.0), 100.0);
    }

    #[test]
    fn target_layer_layout_defaults_to_full_percent_size() {
        let layout = TargetLayerLayout::default();
        assert_eq!(layout.width, DimensionValue::Percent(100.0));
        assert_eq!(layout.height, DimensionValue::Percent(100.0));
    }

    #[test]
    fn target_graph_planner_links_viewport_to_window_root() {
        let targets = HashMap::from([
            (TargetId(1), (TargetKind::Window, Some(7))),
            (TargetId(2), (TargetKind::WidgetRealmViewport, None)),
        ]);
        let layers = HashMap::from([
            (
                (3, TargetId(2)),
                TargetLayerState {
                    realm_id: 3,
                    target_id: TargetId(2),
                    layout: TargetLayerLayout::default(),
                    camera_id: None,
                    environment_id: None,
                },
            ),
            (
                (3, TargetId(1)),
                TargetLayerState {
                    realm_id: 3,
                    target_id: TargetId(1),
                    layout: TargetLayerLayout::default(),
                    camera_id: None,
                    environment_id: None,
                },
            ),
        ]);
        let realms = HashSet::from([RealmId(3)]);

        let plan = TargetGraphPlanner.build_plan(&targets, &layers, &realms);
        assert_eq!(plan.edges.len(), 1);
        assert_eq!(plan.edges[0].parent, TargetId(1));
        assert_eq!(plan.edges[0].child, TargetId(2));
    }
}
