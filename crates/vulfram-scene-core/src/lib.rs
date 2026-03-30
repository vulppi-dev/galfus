use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};

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
}
