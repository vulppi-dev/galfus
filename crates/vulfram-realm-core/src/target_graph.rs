use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    RealmId, TargetEdge, TargetGraphPlan, TargetGraphPlanner, TargetId, TargetKind,
    TargetLayerState,
};

impl TargetGraphPlanner {
    pub fn build_plan(
        &self,
        targets: &HashMap<TargetId, (TargetKind, Option<u32>)>,
        _layers: &HashMap<(u32, TargetId), TargetLayerState>,
        _realms: &HashSet<RealmId>,
    ) -> TargetGraphPlan {
        let mut edges: Vec<TargetEdge> = Vec::new();
        for (_target_id, (kind, _window_id)) in targets {
            let _ = kind;
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
