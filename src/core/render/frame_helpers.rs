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
    vulfram_render::should_render_realm(&mut entry.value, frame_index)
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
    let target_kinds = targets
        .entries
        .iter()
        .map(|(target_id, target)| (*target_id, (target.kind, target.size)))
        .collect();
    vulfram_render::build_target_surface_map(&target_kinds, auto_links)
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
    let targets = universal
        .targets
        .entries
        .iter()
        .map(|(target_id, target)| (*target_id, (target.window_id, target.size)))
        .collect();
    vulfram_render::collect_window_camera_target_sizes(
        &universal.target_layers.entries,
        &targets,
        realm_id,
        window_id,
        window_size,
    )
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
    use super::build_soft_cut_diagnostic;

    #[test]
    fn build_soft_cut_diagnostic_reports_new_cut_set() {
        let frame_report = crate::core::realm::FrameReport {
            cut_edges: vec![crate::core::realm::FrameCutEdge {
                from: 1,
                to: 2,
                connector_id: Some(9),
            }],
            ..Default::default()
        };

        let diagnostic = build_soft_cut_diagnostic(&frame_report, 0, 42);
        assert_eq!(
            diagnostic.as_deref(),
            Some("frame=42 cut_edges=1 connectors=9")
        );
    }
}
