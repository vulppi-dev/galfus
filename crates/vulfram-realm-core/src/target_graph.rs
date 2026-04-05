use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    RealmId, TargetEdge, TargetGraphPlan, TargetGraphPlanner, TargetId, TargetKind,
    TargetLayerState,
};

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
