use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

use vulfram_scene_core::{ConnectorId, RealmGraphPlan, RealmId, SurfaceCache, SurfaceId};

pub fn collect_cut_connectors(plan: &RealmGraphPlan) -> HashSet<ConnectorId> {
    plan.cut_edges
        .iter()
        .filter_map(|edge| edge.connector_id)
        .collect()
}

pub fn update_surface_cache(
    surface_cache: &mut SurfaceCache,
    connectors: &[(ConnectorId, SurfaceId)],
) {
    for (connector_id, source_surface) in connectors {
        surface_cache
            .last_good
            .insert(*connector_id, *source_surface);
        surface_cache
            .fallback
            .entry(*connector_id)
            .or_insert(*source_surface);
    }
}

pub fn collect_connectors_by_realm(
    connectors: &[(ConnectorId, RealmId)],
) -> HashMap<RealmId, Vec<ConnectorId>> {
    let mut map: HashMap<RealmId, Vec<ConnectorId>> = HashMap::new();
    for (connector_id, realm_id) in connectors {
        map.entry(*realm_id).or_default().push(*connector_id);
    }
    for connectors in map.values_mut() {
        connectors.sort_by_key(|id| id.0);
    }
    map
}

pub fn resolve_realm_surface(
    realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
    realm_id: RealmId,
) -> Option<SurfaceId> {
    realm_output_surfaces.get(&realm_id).copied().flatten()
}

pub fn map_realms_to_windows(
    existing_realms: &HashSet<RealmId>,
    layer_windows: &[(RealmId, u32)],
    presents: &[(SurfaceId, u32)],
    realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
) -> HashMap<RealmId, u32> {
    let mut map = HashMap::new();
    for (realm_id, window_id) in layer_windows {
        if !existing_realms.contains(realm_id) {
            continue;
        }
        match map.get_mut(realm_id) {
            Some(existing_window_id) => {
                if *window_id < *existing_window_id {
                    *existing_window_id = *window_id;
                }
            }
            None => {
                map.insert(*realm_id, *window_id);
            }
        }
    }

    let mut surface_to_realm = HashMap::new();
    for (realm_id, surface_id) in realm_output_surfaces {
        if let Some(surface_id) = surface_id {
            surface_to_realm.insert(*surface_id, *realm_id);
        }
    }
    for (surface_id, window_id) in presents {
        if let Some(realm_id) = surface_to_realm.get(surface_id) {
            map.entry(*realm_id).or_insert(*window_id);
        }
    }
    map
}

pub fn update_present_size_cache(
    presents: &[(SurfaceId, u32)],
    window_sizes: &HashMap<u32, glam::UVec2>,
    cache: &mut HashMap<SurfaceId, glam::UVec2>,
    cache_hash: &mut u64,
) -> bool {
    let mut chosen_windows: HashMap<SurfaceId, u32> = HashMap::new();
    for (surface_id, window_id) in presents {
        chosen_windows
            .entry(*surface_id)
            .and_modify(|current_window_id| {
                if *window_id < *current_window_id {
                    *current_window_id = *window_id;
                }
            })
            .or_insert(*window_id);
    }

    let mut aggregate_hash = 0_u64;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut changed = false;

    for (surface_id, window_id) in &chosen_windows {
        let size = window_sizes
            .get(window_id)
            .copied()
            .unwrap_or_else(|| glam::UVec2::new(0, 0));
        if cache.get(surface_id).copied() != Some(size) {
            cache.insert(*surface_id, size);
            changed = true;
        }
        hasher.write_u32(surface_id.0);
        hasher.write_u32(*window_id);
        hasher.write_u32(size.x);
        hasher.write_u32(size.y);
        aggregate_hash ^= hasher.finish();
        hasher = std::collections::hash_map::DefaultHasher::new();
    }

    let previous_len = cache.len();
    cache.retain(|surface_id, _| chosen_windows.contains_key(surface_id));
    if cache.len() != previous_len {
        changed = true;
    }

    if !changed && aggregate_hash == *cache_hash {
        return false;
    }

    *cache_hash = aggregate_hash;
    true
}

#[cfg(test)]
mod tests {
    use super::{
        collect_connectors_by_realm, collect_cut_connectors, map_realms_to_windows,
        resolve_realm_surface, update_present_size_cache, update_surface_cache,
    };
    use std::collections::{HashMap, HashSet};
    use vulfram_scene_core::{
        ConnectorId, RealmGraphEdge, RealmGraphPlan, RealmId, SurfaceCache, SurfaceId,
    };

    #[test]
    fn collects_cut_connectors_from_plan() {
        let plan = RealmGraphPlan {
            order: vec![RealmId(1), RealmId(2)],
            cut_edges: vec![
                RealmGraphEdge {
                    from: RealmId(1),
                    to: RealmId(2),
                    connector_id: Some(ConnectorId(9)),
                },
                RealmGraphEdge {
                    from: RealmId(2),
                    to: RealmId(1),
                    connector_id: None,
                },
            ],
        };

        let cut = collect_cut_connectors(&plan);
        assert!(cut.contains(&ConnectorId(9)));
        assert_eq!(cut.len(), 1);
    }

    #[test]
    fn updates_surface_cache_without_overwriting_fallback() {
        let mut cache = SurfaceCache::default();
        cache.fallback.insert(ConnectorId(4), SurfaceId(99));

        update_surface_cache(
            &mut cache,
            &[
                (ConnectorId(4), SurfaceId(10)),
                (ConnectorId(7), SurfaceId(20)),
            ],
        );

        assert_eq!(cache.last_good.get(&ConnectorId(4)), Some(&SurfaceId(10)));
        assert_eq!(cache.fallback.get(&ConnectorId(4)), Some(&SurfaceId(99)));
        assert_eq!(cache.fallback.get(&ConnectorId(7)), Some(&SurfaceId(20)));
    }

    #[test]
    fn maps_realms_to_smallest_window_and_present_fallback() {
        let existing_realms = HashSet::from([RealmId(1), RealmId(2)]);
        let realm_output_surfaces = HashMap::from([
            (RealmId(1), Some(SurfaceId(50))),
            (RealmId(2), Some(SurfaceId(60))),
        ]);

        let map = map_realms_to_windows(
            &existing_realms,
            &[(RealmId(1), 5), (RealmId(1), 3)],
            &[(SurfaceId(60), 8)],
            &realm_output_surfaces,
        );

        assert_eq!(map.get(&RealmId(1)), Some(&3));
        assert_eq!(map.get(&RealmId(2)), Some(&8));
    }

    #[test]
    fn updates_present_size_cache_and_reports_stability() {
        let presents = vec![(SurfaceId(1), 10), (SurfaceId(2), 11)];
        let window_sizes = HashMap::from([
            (10, glam::UVec2::new(640, 480)),
            (11, glam::UVec2::new(800, 600)),
        ]);
        let mut cache = HashMap::new();
        let mut cache_hash = 0;

        assert!(update_present_size_cache(
            &presents,
            &window_sizes,
            &mut cache,
            &mut cache_hash,
        ));
        assert!(!update_present_size_cache(
            &presents,
            &window_sizes,
            &mut cache,
            &mut cache_hash,
        ));
        assert_eq!(cache.get(&SurfaceId(1)), Some(&glam::UVec2::new(640, 480)));
    }

    #[test]
    fn groups_connectors_and_resolves_surface() {
        let grouped = collect_connectors_by_realm(&[
            (ConnectorId(2), RealmId(9)),
            (ConnectorId(1), RealmId(9)),
            (ConnectorId(5), RealmId(3)),
        ]);
        assert_eq!(
            grouped.get(&RealmId(9)),
            Some(&vec![ConnectorId(1), ConnectorId(2)])
        );

        let realm_output_surfaces = HashMap::from([(RealmId(3), Some(SurfaceId(77)))]);
        assert_eq!(
            resolve_realm_surface(&realm_output_surfaces, RealmId(3)),
            Some(SurfaceId(77))
        );
    }
}
