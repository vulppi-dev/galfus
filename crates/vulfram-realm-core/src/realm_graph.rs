use std::collections::{HashMap, HashSet, VecDeque};

use crate::RealmId;
use vulfram_types::{ConnectorId, SurfaceId};

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
