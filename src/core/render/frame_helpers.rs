use super::RenderState;
use crate::core::realm::RealmId;
use crate::core::state::EngineState;
use crate::core::target::{TargetId, TargetTable};

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

    let layers: Vec<_> = universal
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
        .map(|layer| vulfram_render::EnvironmentLayerBinding {
            target_id: layer.target_id,
            camera_id: layer.camera_id,
            environment_id: layer.environment_id,
            z_index: layer.layout.z_index,
        })
        .collect();

    let plan = vulfram_render::plan_realm_environment_bindings(&layers);

    if let Some(environment_id) = plan.realm_environment_id {
        if let Some(profile) = universal.environment_profiles.get(&environment_id).cloned() {
            render_state.environment = profile;
        }
    }
    for (camera_id, environment_id) in plan.camera_environment_ids {
        let Some(profile) = universal.environment_profiles.get(&environment_id).cloned() else {
            continue;
        };
        render_state
            .camera_environment_overrides
            .insert(camera_id, profile);
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

    let update_requests: Vec<_> = requests
        .iter()
        .filter_map(|(target_id, requested_size)| {
            let target_id = TargetId(*target_id);
            let target = engine_state
                .universal_state
                .targets
                .entries
                .get(&target_id)?;
            Some(vulfram_render::TargetSizeUpdateRequest {
                target_id,
                kind: target.kind,
                current_size: target.size,
                requested_size: *requested_size,
                msaa_samples: target.msaa_samples,
                window_id: target.window_id,
            })
        })
        .collect();

    for update in vulfram_render::plan_target_size_updates(&update_requests) {
        let Some(target) = engine_state
            .universal_state
            .targets
            .entries
            .get_mut(&update.target_id)
        else {
            continue;
        };

        if update.needs_size_update {
            target.size = Some(update.desired_size);
        }

        if update.needs_msaa_init {
            let msaa = update
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
        let next_sources: Vec<_> = target_texture_binds
            .iter()
            .filter_map(|(texture_id, binding)| {
                let surface_id = target_surfaces.get(&binding.target_id)?;
                let surface_target = surface_targets.get(surface_id)?;
                Some(vulfram_render::ExternalTextureSource {
                    texture_id: *texture_id,
                    source_key: surface_target as *const crate::core::resources::RenderTarget
                        as usize,
                })
            })
            .collect();
        let plan = vulfram_render::plan_external_texture_refresh(
            &render_state.external_texture_sources,
            &next_sources,
        );

        for texture_id in plan.replace_ids {
            let Some(binding) = target_texture_binds.get(&texture_id) else {
                continue;
            };
            let Some(surface_id) = target_surfaces.get(&binding.target_id) else {
                continue;
            };
            let Some(surface_target) = surface_targets.get(surface_id) else {
                continue;
            };
            let source_key = surface_target as *const crate::core::resources::RenderTarget as usize;
            render_state
                .external_textures
                .insert(texture_id, surface_target.view.clone());
            render_state
                .external_texture_sources
                .insert(texture_id, source_key);
        }
        for texture_id in plan.stale_ids {
            render_state.external_textures.remove(&texture_id);
            render_state.external_texture_sources.remove(&texture_id);
        }
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
    vulfram_render::build_soft_cut_diagnostic(
        &frame_report.cut_edges,
        previous_cut_edges,
        frame_index,
    )
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
