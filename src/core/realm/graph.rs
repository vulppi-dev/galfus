use std::collections::{HashMap, HashSet, VecDeque};

use super::{ConnectorId, RealmId, UniversalState};

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
    pub fn build_plan(&self, universal: &UniversalState) -> RealmGraphPlan {
        let mut edges = Vec::new();
        let mut hard_targets = HashSet::new();

        for present in universal.presents.entries.values() {
            if let Some(realm_id) = find_realm_by_surface(universal, present.value.surface) {
                hard_targets.insert(realm_id);
            }
        }

        for (connector_id, connector) in universal.connectors.entries.iter() {
            if let Some(source_realm) =
                find_realm_by_surface(universal, connector.value.source_surface)
            {
                edges.push(RealmGraphEdge {
                    from: source_realm,
                    to: connector.value.target_realm,
                    connector_id: Some(*connector_id),
                });
            }
        }

        let mut all_realms: HashSet<RealmId> = universal.realms.entries.keys().copied().collect();
        all_realms.extend(hard_targets.iter().copied());

        let plan_edges: Vec<_> = edges.into_iter().collect();

        let (order, cut_edges) = topo_with_soft_cuts(&all_realms, &plan_edges);

        RealmGraphPlan { order, cut_edges }
    }
}

fn find_realm_by_surface(universal: &UniversalState, surface: super::SurfaceId) -> Option<RealmId> {
    universal
        .realms
        .entries
        .iter()
        .find_map(|(realm_id, entry)| {
            if entry.value.output_surface == Some(surface) {
                Some(*realm_id)
            } else {
                None
            }
        })
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
        for edge in remaining_edges.into_iter() {
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
    for edge in edges {
        if realms.contains(&edge.to) {
            *incoming.entry(edge.to).or_insert(0) += 1;
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
        for edge in edges {
            if edge.from == node {
                if let Some(entry) = incoming.get_mut(&edge.to) {
                    *entry = entry.saturating_sub(1);
                    if *entry == 0 {
                        queue.push_back(edge.to);
                    }
                }
            }
        }
    }

    order
}
