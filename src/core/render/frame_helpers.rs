use super::RenderState;
use crate::core::realm::RealmId;
use crate::core::state::EngineState;
use crate::core::target::{TargetId, TargetKind, TargetTable};

pub(super) fn apply_realm_environment_bindings(
    render_state: &mut RenderState,
    universal: &crate::core::realm::UniversalState,
    realm_id: RealmId,
    window_id: u32,
) {
    render_state.camera_environment_overrides.clear();

    let default_environment = universal
        .default_environment_id
        .and_then(|environment_id| universal.environment_profiles.get(&environment_id))
        .cloned()
        .unwrap_or_default();
    render_state.environment = default_environment;
    render_state.environment_is_configured = true;

    let mut layers: Vec<_> = universal
        .target_layers
        .entries
        .values()
        .filter(|layer| {
            if layer.realm_id != realm_id.0 {
                return false;
            }
            let Some(target) = universal.targets.entries.get(&layer.target_id) else {
                return false;
            };
            target.window_id == Some(window_id)
        })
        .collect();
    layers.sort_by_key(|layer| (layer.layout.z_index, layer.target_id.0));

    for layer in layers {
        let Some(environment_id) = layer.environment_id else {
            continue;
        };
        let Some(profile) = universal.environment_profiles.get(&environment_id).cloned() else {
            continue;
        };
        if let Some(camera_id) = layer.camera_id {
            render_state
                .camera_environment_overrides
                .insert(camera_id, profile);
        } else {
            render_state.environment = profile;
        }
    }
}

pub(super) fn should_render_realm(
    entry: &mut crate::core::realm::TableEntry<crate::core::realm::RealmState>,
    frame_index: u64,
) -> bool {
    let importance = entry.value.importance;
    if importance == 0 {
        return false;
    }
    let base_interval: u64 = match importance {
        1 => 1,
        2 => 2,
        3 => 4,
        _ => 1,
    };
    let cache_multiplier: u64 = match entry.value.cache_policy {
        0 => 1,
        1 => 2,
        2 => 4,
        _ => 1,
    };
    let interval = base_interval.saturating_mul(cache_multiplier);
    let should_render = frame_index.saturating_sub(entry.value.last_render_frame) >= interval;
    if should_render {
        entry.value.last_render_frame = frame_index;
    }
    should_render
}

pub(super) fn write_gpu_timestamp(
    encoder: &mut wgpu::CommandEncoder,
    gpu_profiler: Option<&crate::core::profiling::gpu::GpuProfiler>,
    index: u32,
    gpu_written: &mut bool,
) {
    if let Some(profiler) = gpu_profiler {
        encoder.write_timestamp(profiler.query_set(), index);
        *gpu_written = true;
    }
}

pub(super) fn apply_target_size_requests(
    engine_state: &mut EngineState,
    requests: &std::collections::HashMap<u64, glam::UVec2>,
) {
    if requests.is_empty() {
        return;
    }

    for (target_id, size) in requests {
        let target_id = TargetId(*target_id);
        let Some(target) = engine_state
            .universal_state
            .targets
            .entries
            .get_mut(&target_id)
        else {
            continue;
        };
        if target.kind == TargetKind::Window {
            continue;
        }

        let desired = glam::UVec2::new(size.x.max(1), size.y.max(1));
        if target.size != Some(desired) {
            target.size = Some(desired);
        }

        if target.msaa_samples.is_none() {
            let msaa = target
                .window_id
                .and_then(|window_id| engine_state.render.get(&window_id))
                .map(|state| {
                    engine_state
                        .device
                        .as_ref()
                        .map(|device| {
                            state.msaa_sample_count_for_format(
                                device,
                                wgpu::TextureFormat::Rgba16Float,
                            )
                        })
                        .unwrap_or(1)
                })
                .unwrap_or(1);
            target.msaa_samples = Some(msaa);
        }
    }
}

pub(super) fn build_target_surface_map(
    targets: &TargetTable,
    auto_links: &std::collections::HashMap<(u32, TargetId), crate::core::realm::AutoLink>,
) -> std::collections::HashMap<TargetId, crate::core::realm::SurfaceId> {
    let mut chosen: std::collections::HashMap<TargetId, (u32, crate::core::realm::SurfaceId)> =
        std::collections::HashMap::new();

    for ((realm_id, target_id), link) in auto_links {
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Texture {
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
        .map(|(target_id, (_, surface_id))| (target_id, surface_id))
        .collect()
}

pub(super) fn refresh_window_target_textures(
    render_states: &mut std::collections::HashMap<u32, RenderState>,
    target_texture_binds: &std::collections::HashMap<
        u32,
        crate::core::resources::TargetTextureBinding,
    >,
    target_surfaces: &std::collections::HashMap<TargetId, crate::core::realm::SurfaceId>,
    surface_targets: &std::collections::HashMap<
        crate::core::realm::SurfaceId,
        crate::core::resources::RenderTarget,
    >,
) {
    for render_state in render_states.values_mut() {
        let mut alive_ids = std::collections::HashSet::with_capacity(target_texture_binds.len());
        for (texture_id, binding) in target_texture_binds {
            let Some(surface_id) = target_surfaces.get(&binding.target_id) else {
                continue;
            };
            let Some(surface_target) = surface_targets.get(surface_id) else {
                continue;
            };
            let source_ptr = surface_target as *const crate::core::resources::RenderTarget as usize;
            alive_ids.insert(*texture_id);
            let needs_replace = render_state
                .external_texture_sources
                .get(texture_id)
                .copied()
                != Some(source_ptr);
            if needs_replace {
                render_state
                    .external_textures
                    .insert(*texture_id, surface_target.view.clone());
                render_state
                    .external_texture_sources
                    .insert(*texture_id, source_ptr);
            }
        }
        render_state
            .external_textures
            .retain(|texture_id, _| alive_ids.contains(texture_id));
        render_state
            .external_texture_sources
            .retain(|texture_id, _| alive_ids.contains(texture_id));
    }
}

pub(super) fn collect_window_camera_target_sizes(
    universal: &crate::core::realm::UniversalState,
    realm_id: crate::core::realm::RealmId,
    window_id: u32,
    window_size: glam::UVec2,
) -> std::collections::HashMap<u32, glam::UVec2> {
    const DEFAULT_CH_WIDTH: f32 = 8.0;
    let mut sizes = std::collections::HashMap::new();
    for layer in universal.target_layers.entries.values() {
        if layer.realm_id != realm_id.0 {
            continue;
        }
        let Some(camera_id) = layer.camera_id else {
            continue;
        };
        let Some(target) = universal.targets.entries.get(&layer.target_id) else {
            continue;
        };
        if target.window_id != Some(window_id) {
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

        let size = target
            .size
            .unwrap_or(glam::UVec2::new(layout_width, layout_height));
        sizes.insert(camera_id, glam::UVec2::new(size.x.max(1), size.y.max(1)));
    }
    sizes
}

pub(super) fn build_soft_cut_diagnostic(
    frame_report: &crate::core::realm::FrameReport,
    previous_cut_edges: usize,
    frame_index: u64,
) -> Option<String> {
    if frame_report.cut_edges.is_empty()
        || !(previous_cut_edges == 0 || previous_cut_edges != frame_report.cut_edges.len())
    {
        return None;
    }
    let cut_count = frame_report.cut_edges.len();
    let connectors: Vec<u32> = frame_report
        .cut_edges
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
        frame_index, cut_count, connector_text
    ))
}

#[cfg(test)]
mod tests {
    use super::collect_window_camera_target_sizes;
    use crate::core::realm::RealmId;
    use crate::core::target::{
        DimensionValue, TargetId, TargetKind, TargetLayerLayout, TargetLayerState, TargetState,
    };

    #[test]
    fn camera_target_size_uses_layer_layout_when_target_has_no_fixed_size() {
        let mut universal = crate::core::realm::UniversalState::default();
        let target_id = TargetId(100);
        universal.targets.entries.insert(
            target_id,
            TargetState {
                kind: TargetKind::Window,
                window_id: Some(9),
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        universal.target_layers.entries.insert(
            (77, target_id),
            TargetLayerState {
                realm_id: 77,
                target_id,
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
        );

        let sizes = collect_window_camera_target_sizes(
            &universal,
            RealmId(77),
            9,
            glam::UVec2::new(1920, 1080),
        );
        assert_eq!(sizes.get(&501), Some(&glam::UVec2::new(960, 270)));
    }

    #[test]
    fn camera_target_size_prefers_explicit_target_size() {
        let mut universal = crate::core::realm::UniversalState::default();
        let target_id = TargetId(101);
        universal.targets.entries.insert(
            target_id,
            TargetState {
                kind: TargetKind::Window,
                window_id: Some(9),
                size: Some(glam::UVec2::new(333, 222)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        universal.target_layers.entries.insert(
            (77, target_id),
            TargetLayerState {
                realm_id: 77,
                target_id,
                layout: TargetLayerLayout {
                    left: DimensionValue::Percent(0.0),
                    top: DimensionValue::Percent(0.0),
                    width: DimensionValue::Percent(50.0),
                    height: DimensionValue::Percent(25.0),
                    z_index: 0,
                    blend_mode: 0,
                    clip: None,
                },
                camera_id: Some(777),
                environment_id: None,
            },
        );

        let sizes = collect_window_camera_target_sizes(
            &universal,
            RealmId(77),
            9,
            glam::UVec2::new(1920, 1080),
        );
        assert_eq!(sizes.get(&777), Some(&glam::UVec2::new(333, 222)));
    }
}
