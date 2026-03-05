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
        let surface_to_realm = collect_surface_to_realm(universal);

        for present in universal.presents.entries.values() {
            if let Some(realm_id) = surface_to_realm.get(&present.value.surface).copied() {
                hard_targets.insert(realm_id);
            }
        }

        for (connector_id, connector) in universal.connectors.entries.iter() {
            if let Some(source_realm) = surface_to_realm
                .get(&connector.value.source_surface)
                .copied()
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

fn collect_surface_to_realm(universal: &UniversalState) -> HashMap<super::SurfaceId, RealmId> {
    let mut map = HashMap::new();
    for (realm_id, entry) in &universal.realms.entries {
        if let Some(surface_id) = entry.value.output_surface {
            map.insert(surface_id, *realm_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::realm::{
        ConnectorState, PresentState, RealmKind, RealmState, SurfaceKind, SurfaceState,
    };

    fn realm_state(output_surface: Option<crate::core::realm::SurfaceId>) -> RealmState {
        RealmState {
            kind: RealmKind::TwoD,
            output_surface,
            render_graph: None,
            importance: 1,
            cache_policy: 0,
            last_render_frame: 0,
        }
    }

    #[test]
    fn planner_orders_linear_dependency() {
        let mut universal = UniversalState::default();
        let surface_a = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let surface_b = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let realm_a = universal.realms.alloc(realm_state(Some(surface_a)));
        let realm_b = universal.realms.alloc(realm_state(Some(surface_b)));

        let _connector = universal.connectors.alloc(ConnectorState {
            target_realm: realm_b,
            source_surface: surface_a,
            rect: glam::Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        });
        let _present = universal.presents.alloc(PresentState {
            window_id: 1,
            surface: surface_b,
        });

        let plan = RealmGraphPlanner.build_plan(&universal);
        assert_eq!(plan.order, vec![realm_a, realm_b]);
        assert!(plan.cut_edges.is_empty());
    }

    #[test]
    fn planner_cuts_cycle_and_keeps_deterministic_order() {
        let mut universal = UniversalState::default();
        let surface_a = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let surface_b = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let realm_a = universal.realms.alloc(realm_state(Some(surface_a)));
        let realm_b = universal.realms.alloc(realm_state(Some(surface_b)));

        let _ab = universal.connectors.alloc(ConnectorState {
            target_realm: realm_b,
            source_surface: surface_a,
            rect: glam::Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        });
        let _ba = universal.connectors.alloc(ConnectorState {
            target_realm: realm_a,
            source_surface: surface_b,
            rect: glam::Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        });

        let plan = RealmGraphPlanner.build_plan(&universal);
        assert_eq!(plan.order, vec![realm_a, realm_b]);
        assert_eq!(plan.cut_edges.len(), 2);
    }

    #[test]
    fn planner_includes_disconnected_realms() {
        let mut universal = UniversalState::default();
        let surface_a = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(16, 16),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let realm_a = universal.realms.alloc(realm_state(Some(surface_a)));
        let realm_b = universal.realms.alloc(realm_state(None));

        let plan = RealmGraphPlanner.build_plan(&universal);
        assert_eq!(plan.order, vec![realm_a, realm_b]);
        assert!(plan.cut_edges.is_empty());
    }
}
