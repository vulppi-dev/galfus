use std::collections::{HashMap, HashSet, VecDeque};

use super::{ConnectorId, RealmId, UniversalState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmGraphEdgeKind {
    Hard,
    Soft,
}

#[derive(Debug, Clone)]
pub struct RealmGraphEdge {
    pub from: RealmId,
    pub to: RealmId,
    pub kind: RealmGraphEdgeKind,
    pub connector_id: Option<ConnectorId>,
}

#[derive(Debug, Default)]
pub struct RealmGraphPlan {
    pub order: Vec<RealmId>,
    pub edges: Vec<RealmGraphEdge>,
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
            if let Some(source_realm) = find_realm_by_surface(universal, connector.value.source_surface)
            {
                edges.push(RealmGraphEdge {
                    from: source_realm,
                    to: connector.value.target_realm,
                    kind: RealmGraphEdgeKind::Soft,
                    connector_id: Some(*connector_id),
                });
            }
        }

        let mut all_realms: HashSet<RealmId> = universal.realms.entries.keys().copied().collect();
        all_realms.extend(hard_targets.iter().copied());

        let mut plan_edges = Vec::new();
        plan_edges.extend(edges.into_iter());

        let (order, cut_edges) = topo_with_soft_cuts(&all_realms, &plan_edges);

        RealmGraphPlan {
            order,
            edges: plan_edges,
            cut_edges,
        }
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

    let mut order = Vec::new();
    let remaining_edges: Vec<RealmGraphEdge> = edges.to_vec();
    let mut cut_edges = Vec::new();

    while let Some(node) = queue.pop_front() {
        order.push(node);
        let mut i = 0;
        while i < remaining_edges.len() {
            if remaining_edges[i].from == node {
                let to = remaining_edges[i].to;
                if let Some(entry) = incoming.get_mut(&to) {
                    *entry = entry.saturating_sub(1);
                    if *entry == 0 {
                        queue.push_back(to);
                    }
                }
            }
            i += 1;
        }
    }

    if order.len() != realms.len() {
        let mut unresolved: HashSet<RealmId> = realms.iter().copied().collect();
        for node in &order {
            unresolved.remove(node);
        }
        let mut pruned = Vec::new();
        for edge in remaining_edges.into_iter() {
            if unresolved.contains(&edge.from) && unresolved.contains(&edge.to) {
                if edge.kind == RealmGraphEdgeKind::Soft {
                    cut_edges.push(edge);
                } else {
                    pruned.push(edge);
                }
            } else {
                pruned.push(edge);
            }
        }
        let (order_retry, mut cut_retry) = topo_with_soft_cuts(&unresolved, &pruned);
        order.extend(order_retry);
        cut_edges.append(&mut cut_retry);
    }

    (order, cut_edges)
}
