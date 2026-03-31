use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

use vulfram_realm_core::{
    AutoLink, ConnectorId, FrameCutEdge, RealmGraphPlan, RealmId, RealmState, SurfaceCache,
    SurfaceId, TargetId, TargetKind, TargetLayerState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentLayerBinding {
    pub target_id: TargetId,
    pub camera_id: Option<u32>,
    pub environment_id: Option<u32>,
    pub z_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmEnvironmentBindingPlan {
    pub realm_environment_id: Option<u32>,
    pub camera_environment_ids: HashMap<u32, u32>,
}

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

pub fn should_render_realm(realm_state: &mut RealmState, frame_index: u64) -> bool {
    let importance = realm_state.importance;
    if importance == 0 {
        return false;
    }
    let base_interval: u64 = match importance {
        1 => 1,
        2 => 2,
        3 => 4,
        _ => 1,
    };
    let cache_multiplier: u64 = match realm_state.cache_policy {
        0 => 1,
        1 => 2,
        2 => 4,
        _ => 1,
    };
    let interval = base_interval.saturating_mul(cache_multiplier);
    let should_render = frame_index.saturating_sub(realm_state.last_render_frame) >= interval;
    if should_render {
        realm_state.last_render_frame = frame_index;
    }
    should_render
}

pub fn build_target_surface_map(
    targets: &HashMap<TargetId, (TargetKind, Option<glam::UVec2>)>,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
) -> HashMap<TargetId, SurfaceId> {
    let mut chosen: HashMap<TargetId, (u32, SurfaceId)> = HashMap::new();

    for ((realm_id, target_id), link) in auto_links {
        let Some((kind, _size)) = targets.get(target_id) else {
            continue;
        };
        if *kind != TargetKind::Texture {
            continue;
        }

        match chosen.get(target_id) {
            Some((current_realm, _)) if *current_realm <= *realm_id => {}
            _ => {
                chosen.insert(*target_id, (*realm_id, link.surface_id));
            }
        }
    }

    chosen
        .into_iter()
        .map(|(target_id, (_realm_id, surface_id))| (target_id, surface_id))
        .collect()
}

pub fn collect_window_camera_target_sizes(
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    targets: &HashMap<TargetId, (Option<u32>, Option<glam::UVec2>)>,
    realm_id: RealmId,
    window_id: u32,
    window_size: glam::UVec2,
) -> HashMap<u32, glam::UVec2> {
    const DEFAULT_CH_WIDTH: f32 = 8.0;
    let mut sizes = HashMap::new();
    for layer in layers.values() {
        if layer.realm_id != realm_id.0 {
            continue;
        }
        let Some(camera_id) = layer.camera_id else {
            continue;
        };
        let Some((target_window_id, target_size)) = targets.get(&layer.target_id) else {
            continue;
        };
        if *target_window_id != Some(window_id) {
            continue;
        }

        let ref_width = window_size.x.max(1) as f32;
        let ref_height = window_size.y.max(1) as f32;
        let layout_width = layer
            .layout
            .width
            .resolve(ref_width, DEFAULT_CH_WIDTH)
            .max(1.0)
            .round() as u32;
        let layout_height = layer
            .layout
            .height
            .resolve(ref_height, DEFAULT_CH_WIDTH)
            .max(1.0)
            .round() as u32;

        let size = target_size.unwrap_or(glam::UVec2::new(layout_width, layout_height));
        sizes.insert(camera_id, glam::UVec2::new(size.x.max(1), size.y.max(1)));
    }
    sizes
}

pub fn plan_realm_environment_bindings(
    layers: &[EnvironmentLayerBinding],
) -> RealmEnvironmentBindingPlan {
    let mut ordered_layers = layers.to_vec();
    ordered_layers.sort_by_key(|layer| (layer.z_index, layer.target_id.0));

    let mut realm_environment_id = None;
    let mut camera_environment_ids = HashMap::new();
    for layer in ordered_layers {
        let Some(environment_id) = layer.environment_id else {
            continue;
        };
        if let Some(camera_id) = layer.camera_id {
            camera_environment_ids.insert(camera_id, environment_id);
        } else {
            realm_environment_id = Some(environment_id);
        }
    }

    RealmEnvironmentBindingPlan {
        realm_environment_id,
        camera_environment_ids,
    }
}

pub fn build_soft_cut_diagnostic(
    cut_edges: &[FrameCutEdge],
    previous_cut_edges: usize,
    frame_index: u64,
) -> Option<String> {
    if cut_edges.is_empty() || !(previous_cut_edges == 0 || previous_cut_edges != cut_edges.len()) {
        return None;
    }

    let connectors: Vec<_> = cut_edges
        .iter()
        .filter_map(|edge| edge.connector_id)
        .collect();
    let connector_text = if connectors.is_empty() {
        "none".to_string()
    } else {
        connectors
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };

    Some(format!(
        "frame={} cut_edges={} connectors={}",
        frame_index,
        cut_edges.len(),
        connector_text
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        EnvironmentLayerBinding, RealmEnvironmentBindingPlan, build_soft_cut_diagnostic,
        build_target_surface_map, collect_connectors_by_realm, collect_cut_connectors,
        collect_window_camera_target_sizes, map_realms_to_windows, plan_realm_environment_bindings,
        resolve_realm_surface, should_render_realm, update_present_size_cache,
        update_surface_cache,
    };
    use std::collections::{HashMap, HashSet};
    use vulfram_realm_core::{
        AutoLink, ConnectorId, DimensionValue, FrameCutEdge, RealmGraphEdge, RealmGraphPlan,
        RealmId, RealmState, SurfaceCache, SurfaceId, TargetId, TargetKind, TargetLayerLayout,
        TargetLayerState,
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

    #[test]
    fn should_render_realm_tracks_interval_and_updates_frame() {
        let mut realm = RealmState {
            kind: vulfram_realm_core::RealmKind::ThreeD,
            output_surface: None,
            render_graph_id: None,
            importance: 2,
            cache_policy: 1,
            last_render_frame: 0,
        };

        assert!(!should_render_realm(&mut realm, 1));
        assert!(should_render_realm(&mut realm, 4));
        assert_eq!(realm.last_render_frame, 4);
    }

    #[test]
    fn builds_target_surface_map_for_texture_targets() {
        let targets = HashMap::from([
            (TargetId(1), (TargetKind::Texture, None)),
            (TargetId(2), (TargetKind::Window, None)),
        ]);
        let auto_links = HashMap::from([
            (
                (8, TargetId(1)),
                AutoLink {
                    surface_id: SurfaceId(40),
                    connector_id: None,
                    present_id: None,
                },
            ),
            (
                (3, TargetId(1)),
                AutoLink {
                    surface_id: SurfaceId(30),
                    connector_id: None,
                    present_id: None,
                },
            ),
        ]);

        let map = build_target_surface_map(&targets, &auto_links);
        assert_eq!(map.get(&TargetId(1)), Some(&SurfaceId(30)));
        assert!(!map.contains_key(&TargetId(2)));
    }

    #[test]
    fn collects_window_camera_target_sizes_from_layout_or_explicit_size() {
        let layers = HashMap::from([
            (
                (77, TargetId(1)),
                TargetLayerState {
                    realm_id: 77,
                    target_id: TargetId(1),
                    layout: TargetLayerLayout {
                        left: DimensionValue::Percent(0.0),
                        top: DimensionValue::Percent(0.0),
                        width: DimensionValue::Percent(50.0),
                        height: DimensionValue::Percent(25.0),
                        z_index: 0,
                        blend_mode: 0,
                        clip: None,
                    },
                    camera_id: Some(501),
                    environment_id: None,
                },
            ),
            (
                (77, TargetId(2)),
                TargetLayerState {
                    realm_id: 77,
                    target_id: TargetId(2),
                    layout: TargetLayerLayout {
                        left: DimensionValue::Percent(0.0),
                        top: DimensionValue::Percent(0.0),
                        width: DimensionValue::Percent(10.0),
                        height: DimensionValue::Percent(10.0),
                        z_index: 0,
                        blend_mode: 0,
                        clip: None,
                    },
                    camera_id: Some(777),
                    environment_id: None,
                },
            ),
        ]);
        let targets = HashMap::from([
            (TargetId(1), (Some(9), None)),
            (TargetId(2), (Some(9), Some(glam::UVec2::new(333, 222)))),
        ]);

        let sizes = collect_window_camera_target_sizes(
            &layers,
            &targets,
            RealmId(77),
            9,
            glam::UVec2::new(1920, 1080),
        );
        assert_eq!(sizes.get(&501), Some(&glam::UVec2::new(960, 270)));
        assert_eq!(sizes.get(&777), Some(&glam::UVec2::new(333, 222)));
    }

    #[test]
    fn plans_realm_environment_bindings_in_z_order() {
        let plan = plan_realm_environment_bindings(&[
            EnvironmentLayerBinding {
                target_id: TargetId(9),
                camera_id: Some(7),
                environment_id: Some(30),
                z_index: 5,
            },
            EnvironmentLayerBinding {
                target_id: TargetId(2),
                camera_id: None,
                environment_id: Some(11),
                z_index: 0,
            },
            EnvironmentLayerBinding {
                target_id: TargetId(3),
                camera_id: None,
                environment_id: Some(12),
                z_index: 10,
            },
        ]);

        assert_eq!(
            plan,
            RealmEnvironmentBindingPlan {
                realm_environment_id: Some(12),
                camera_environment_ids: HashMap::from([(7, 30)]),
            }
        );
    }

    #[test]
    fn soft_cut_diagnostic_reports_connector_summary() {
        let diagnostic = build_soft_cut_diagnostic(
            &[FrameCutEdge {
                from: 1,
                to: 2,
                connector_id: Some(9),
            }],
            0,
            42,
        );

        assert_eq!(
            diagnostic.as_deref(),
            Some("frame=42 cut_edges=1 connectors=9")
        );
    }
}
