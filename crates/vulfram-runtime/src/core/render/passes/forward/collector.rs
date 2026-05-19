use crate::core::render::state::DrawItem;
use crate::core::resources::geometry::Frustum;
use crate::core::resources::{
    CameraRecord, MATERIAL_FALLBACK_ID, PolygonMode, PrimitiveTopology, RenderSide, SurfaceType,
};

pub(crate) fn collect_objects(
    scene: &crate::core::render::state::RenderScene,
    collector: &mut crate::core::render::state::DrawCollector,
    camera_record: &CameraRecord,
    vertex_sys: &crate::core::resources::VertexAllocatorSystem,
    log_events: &mut Vec<vulfram_log::LogEvent>,
) -> u32 {
    let materials = &scene.materials;
    let frustum = Frustum::from_view_projection(camera_record.data.view_projection);

    let mut instance_cursor = 0;
    let mut skipped_layer_mask = 0u32;
    let mut skipped_missing_geometry = 0u32;
    let mut skipped_frustum = 0u32;
    let mut used_fallback_material = 0u32;
    let mut used_missing_material_fallback = 0u32;
    let mut used_invalid_material_fallback = 0u32;
    let mut pbr_models = 0u32;
    let mut standard_models = 0u32;

    for (model_id, model_record) in &scene.models {
        if (model_record.layer_mask & camera_record.layer_mask) == 0 {
            skipped_layer_mask += 1;
            continue;
        }

        if vertex_sys
            .index_info(model_record.geometry_id)
            .ok()
            .flatten()
            .is_none()
        {
            skipped_missing_geometry += 1;
            continue;
        }

        if let Some(aabb) = vertex_sys.aabb(model_record.geometry_id) {
            let has_position = vertex_sys
                .geometry_has_streams(model_record.geometry_id, &[crate::core::resources::VertexStream::Position])
                .unwrap_or(false);
            let has_normal = vertex_sys
                .geometry_has_streams(model_record.geometry_id, &[crate::core::resources::VertexStream::Normal])
                .unwrap_or(false);
            if instance_cursor == 0 {
                vulfram_log::vulfram_log_debug!(
                    log_events,
                    "forward.geometry",
                    "geom={} has_pos={} has_nrm={} aabb_min=({:.3},{:.3},{:.3}) aabb_max=({:.3},{:.3},{:.3})",
                    model_record.geometry_id,
                    has_position,
                    has_normal,
                    aabb.min.x,
                    aabb.min.y,
                    aabb.min.z,
                    aabb.max.x,
                    aabb.max.y,
                    aabb.max.z
                );
            }
            let world_aabb = aabb.transform(&model_record.data.transform);
            if !frustum.intersects_aabb(world_aabb.min, world_aabb.max) {
                skipped_frustum += 1;
                continue;
            }
        }

        let material_id = model_record.material_id.unwrap_or(MATERIAL_FALLBACK_ID);
        if model_record.material_id.is_none() {
            used_fallback_material += 1;
        }

        let model_depth = {
            let clip = camera_record.data.view_projection * model_record.data.translation;
            if *model_id == 5002 {
                let ndc = if clip.w.abs() > 1e-6 {
                    clip / clip.w
                } else {
                    clip
                };
                vulfram_log::vulfram_log_debug!(
                    log_events,
                    "forward.clip",
                    "model={} clip=({:.3},{:.3},{:.3},{:.3}) ndc=({:.3},{:.3},{:.3})",
                    model_id,
                    clip.x,
                    clip.y,
                    clip.z,
                    clip.w,
                    ndc.x,
                    ndc.y,
                    ndc.z
                );
            }
            if clip.w.abs() > 1e-5 {
                clip.z / clip.w
            } else {
                0.0
            }
        };

        let fallback_needed = materials
            .get(&material_id)
            .map(|record| record.compiled_shader_source.is_none() || record.compile_error.is_some())
            .unwrap_or(true);
        let material_id = if fallback_needed {
            if model_record.material_id.is_some() {
                vulfram_log::vulfram_log_warn!(
                    log_events,
                    "material.draw.fallback.standard",
                    "model={} material={} replaced_by_fallback={}",
                    model_id,
                    model_record.material_id.unwrap_or(MATERIAL_FALLBACK_ID),
                    MATERIAL_FALLBACK_ID
                );
            }
            used_invalid_material_fallback += 1;
            MATERIAL_FALLBACK_ID
        } else {
            material_id
        };

        if let Some(record) = materials.get(&material_id)
            && matches!(
                record.base_preset,
                crate::core::resources::ShaderMaterialPreset::Pbr
            )
        {
            pbr_models += 1;
            let item = DrawItem {
                model_id: *model_id,
                geometry_id: model_record.geometry_id,
                material_id,
                compiled_shader_hash: record.compiled_shader_hash,
                topology: record.topology,
                polygon_mode: record.polygon_mode,
                render_side: record.render_side,
                depth: model_depth,
                instance_idx: 0,
            };
            match record.surface_type {
                SurfaceType::Opaque => collector.pbr_opaque.push(item),
                SurfaceType::Masked => collector.pbr_masked.push(item),
                SurfaceType::Transparent => collector.transparent.push(item),
            }
            continue;
        }

        let material_id = model_record
            .material_id
            .filter(|id| materials.contains_key(id))
            .unwrap_or(MATERIAL_FALLBACK_ID);
        if model_record.material_id.is_some() && material_id == MATERIAL_FALLBACK_ID {
            used_missing_material_fallback += 1;
        }

        let (surface_type, topology, polygon_mode, render_side) = materials
            .get(&material_id)
            .map(|record| {
                (
                    record.surface_type,
                    record.topology,
                    record.polygon_mode,
                    record.render_side,
                )
            })
            .unwrap_or((
                SurfaceType::Opaque,
                PrimitiveTopology::TriangleList,
                PolygonMode::Fill,
                RenderSide::Front,
            ));
        standard_models += 1;

        let item = DrawItem {
            model_id: *model_id,
            geometry_id: model_record.geometry_id,
            material_id,
            compiled_shader_hash: materials
                .get(&material_id)
                .map(|record| record.compiled_shader_hash)
                .unwrap_or(0),
            topology,
            polygon_mode,
            render_side,
            depth: model_depth,
            instance_idx: 0,
        };

        match surface_type {
            SurfaceType::Opaque => collector.standard_opaque.push(item),
            SurfaceType::Masked => collector.standard_masked.push(item),
            SurfaceType::Transparent => collector.transparent.push(item),
        }
    }

    // Sort and prepare instance data
    sort_collector(collector);
    let kept_count = collector.pbr_opaque.len()
        + collector.standard_opaque.len()
        + collector.pbr_masked.len()
        + collector.standard_masked.len()
        + collector.transparent.len();
    vulfram_log::vulfram_log_debug!(
        log_events,
        "forward.collector",
        "models_total={} kept={} pbr={} standard={} skip_layer={} skip_geom={} skip_frustum={} fallback_none={} fallback_missing={} fallback_invalid={} pbr_o={} std_o={} pbr_t={} std_t={}",
        scene.models.len(),
        kept_count,
        pbr_models,
        standard_models,
        skipped_layer_mask,
        skipped_missing_geometry,
        skipped_frustum,
        used_fallback_material,
        used_missing_material_fallback,
        used_invalid_material_fallback,
        collector.pbr_opaque.len(),
        collector.standard_opaque.len(),
        collector
            .transparent
            .iter()
            .filter(|item| {
                materials
                    .get(&item.material_id)
                    .map(|record| {
                        matches!(
                            record.base_preset,
                            crate::core::resources::ShaderMaterialPreset::Pbr
                        )
                    })
                    .unwrap_or(false)
            })
            .count(),
        collector
            .transparent
            .iter()
            .filter(|item| {
                materials
                    .get(&item.material_id)
                    .map(|record| {
                        matches!(
                            record.base_preset,
                            crate::core::resources::ShaderMaterialPreset::Standard
                        )
                    })
                    .unwrap_or(true)
            })
            .count()
    );

    let groups = [
        &mut collector.pbr_opaque,
        &mut collector.standard_opaque,
        &mut collector.pbr_masked,
        &mut collector.standard_masked,
        &mut collector.transparent,
    ];

    for group in groups {
        for item in group.iter_mut() {
            item.instance_idx = instance_cursor;
            if let Some(record) = scene.models.get(&item.model_id) {
                collector.instance_data.push(record.data);
                instance_cursor += 1;
            }
        }
    }

    if let Some(first) = collector.instance_data.first() {
        vulfram_log::vulfram_log_debug!(
            log_events,
            "forward.instance",
            "instances={} first_translation=({:.3},{:.3},{:.3},{:.3}) first_scale=({:.3},{:.3},{:.3},{:.3})",
            collector.instance_data.len(),
            first.translation.x,
            first.translation.y,
            first.translation.z,
            first.translation.w,
            first.scale.x,
            first.scale.y,
            first.scale.z,
            first.scale.w
        );
    }

    instance_cursor
}

fn sort_collector(collector: &mut crate::core::render::state::DrawCollector) {
    collector.pbr_opaque.sort_by_key(|a| {
        (
            a.topology as u32,
            a.polygon_mode as u32,
            a.render_side as u32,
            a.compiled_shader_hash,
            a.material_id,
            a.geometry_id,
        )
    });
    collector.standard_opaque.sort_by_key(|a| {
        (
            a.topology as u32,
            a.polygon_mode as u32,
            a.render_side as u32,
            a.compiled_shader_hash,
            a.material_id,
            a.geometry_id,
        )
    });
    collector.pbr_masked.sort_by_key(|a| {
        (
            a.topology as u32,
            a.polygon_mode as u32,
            a.render_side as u32,
            a.compiled_shader_hash,
            a.material_id,
            a.geometry_id,
        )
    });
    collector.standard_masked.sort_by_key(|a| {
        (
            a.topology as u32,
            a.polygon_mode as u32,
            a.render_side as u32,
            a.compiled_shader_hash,
            a.material_id,
            a.geometry_id,
        )
    });

    // Sort Far-to-Near (Painter's Algorithm)
    // With Reverse Z: Far is 0.0, Near is 1.0. So we sort Ascending.
    collector.transparent.sort_by(|a, b| {
        a.depth
            .partial_cmp(&b.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
