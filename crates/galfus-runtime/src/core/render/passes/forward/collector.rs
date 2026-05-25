use crate::core::render::state::DrawItem;
use crate::core::resources::geometry::Frustum;
use crate::core::resources::{
    CameraRecord, MATERIAL_FALLBACK_ID, PolygonMode, PrimitiveTopology, RenderSide, SurfaceType,
};

fn material_allows_3d(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    matches!(record.realm_kind, crate::core::resources::MaterialRealmKind::ThreeD)
}

pub(crate) fn collect_objects(
    scene: &crate::core::render::state::RenderScene,
    collector: &mut crate::core::render::state::DrawCollector,
    camera_record: &CameraRecord,
    vertex_sys: &crate::core::resources::VertexAllocatorSystem,
    log_events: &mut Vec<galfus_log::LogEvent>,
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
    let mut shaded_models = 0u32;

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
                .geometry_has_streams(
                    model_record.geometry_id,
                    &[crate::core::resources::VertexStream::Position],
                )
                .unwrap_or(false);
            let has_normal = vertex_sys
                .geometry_has_streams(
                    model_record.geometry_id,
                    &[crate::core::resources::VertexStream::Normal],
                )
                .unwrap_or(false);
            if instance_cursor == 0 {
                galfus_log::galfus_log_debug!(
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
                galfus_log::galfus_log_debug!(
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

        let material_debug = materials.get(&material_id).map(|record| {
            (
                material_allows_3d(record),
                record.compiled_shader_source.is_some(),
                record.compile_error.clone(),
                record.realm_kind,
                record.base_preset,
            )
        });
        let fallback_needed = material_debug
            .as_ref()
            .map(|(allows_3d, has_compiled, compile_error, _, _)| {
                !*allows_3d || !*has_compiled || compile_error.is_some()
            })
            .unwrap_or(true);
        let resolved_material_id = if fallback_needed {
            if model_record.material_id.is_some() {
                let reason = if let Some((
                    allows_3d,
                    has_compiled,
                    compile_error,
                    realm_kind,
                    base_preset,
                )) = material_debug.as_ref()
                {
                    format!(
                        "allows_3d={} has_compiled={} compile_error={:?} realm_kind={:?} base_preset={:?}",
                        allows_3d, has_compiled, compile_error, realm_kind, base_preset
                    )
                } else {
                    "material_missing".to_string()
                };
                galfus_log::galfus_log_warn!(
                    log_events,
                    "material.draw.fallback.standard",
                    "model={} material={} replaced_by_fallback={} reason={}",
                    model_id,
                    model_record.material_id.unwrap_or(MATERIAL_FALLBACK_ID),
                    MATERIAL_FALLBACK_ID,
                    reason
                );
            }
            used_invalid_material_fallback += 1;
            MATERIAL_FALLBACK_ID
        } else {
            material_id
        };

        let material_id = model_record
            .material_id
            .map(|_| resolved_material_id)
            .filter(|id| materials.contains_key(id))
            .unwrap_or(resolved_material_id);
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
        shaded_models += 1;

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
            SurfaceType::Opaque => collector.opaque.push(item),
            SurfaceType::Masked => collector.masked.push(item),
            SurfaceType::Transparent => collector.transparent.push(item),
        }
    }

    // Sort and prepare instance data
    sort_collector(collector);
    let kept_count = collector.opaque.len() + collector.masked.len() + collector.transparent.len();
    galfus_log::galfus_log_debug!(
        log_events,
        "forward.collector",
        "models_total={} kept={} shaded={} skip_layer={} skip_geom={} skip_frustum={} fallback_none={} fallback_missing={} fallback_invalid={} opaque={} masked={} transparent={}",
        scene.models.len(),
        kept_count,
        shaded_models,
        skipped_layer_mask,
        skipped_missing_geometry,
        skipped_frustum,
        used_fallback_material,
        used_missing_material_fallback,
        used_invalid_material_fallback,
        collector.opaque.len(),
        collector.masked.len(),
        collector.transparent.len()
    );

    let groups = [
        &mut collector.opaque,
        &mut collector.masked,
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
        galfus_log::galfus_log_debug!(
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
    collector.opaque.sort_by_key(|a| {
        (
            a.topology as u32,
            a.polygon_mode as u32,
            a.render_side as u32,
            a.compiled_shader_hash,
            a.material_id,
            a.geometry_id,
        )
    });
    collector.masked.sort_by_key(|a| {
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
            .then_with(|| (a.topology as u32).cmp(&(b.topology as u32)))
            .then_with(|| (a.polygon_mode as u32).cmp(&(b.polygon_mode as u32)))
            .then_with(|| (a.render_side as u32).cmp(&(b.render_side as u32)))
            .then_with(|| a.compiled_shader_hash.cmp(&b.compiled_shader_hash))
            .then_with(|| a.material_id.cmp(&b.material_id))
            .then_with(|| a.geometry_id.cmp(&b.geometry_id))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
}

#[cfg(test)]
mod tests {
    use super::sort_collector;
    use crate::core::render::state::{DrawCollector, DrawItem};
    use crate::core::resources::{PolygonMode, PrimitiveTopology, RenderSide};

    fn item(
        model_id: u32,
        material_id: u32,
        geometry_id: u32,
        shader_hash: u64,
        depth: f32,
    ) -> DrawItem {
        DrawItem {
            model_id,
            geometry_id,
            material_id,
            compiled_shader_hash: shader_hash,
            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Fill,
            render_side: RenderSide::Front,
            depth,
            instance_idx: 0,
        }
    }

    #[test]
    fn sort_collector_orders_opaque_and_masked_by_pipeline_key() {
        let mut collector = DrawCollector::default();
        collector.opaque.push(item(10, 7, 2, 9, 0.3));
        collector.opaque.push(item(11, 3, 1, 5, 0.4));
        collector.masked.push(item(12, 5, 8, 1, 0.5));
        collector.masked.push(item(13, 4, 9, 1, 0.2));

        sort_collector(&mut collector);

        let opaque_order: Vec<(u64, u32, u32)> = collector
            .opaque
            .iter()
            .map(|it| (it.compiled_shader_hash, it.material_id, it.geometry_id))
            .collect();
        assert_eq!(opaque_order, vec![(5, 3, 1), (9, 7, 2)]);

        let masked_order: Vec<(u64, u32, u32)> = collector
            .masked
            .iter()
            .map(|it| (it.compiled_shader_hash, it.material_id, it.geometry_id))
            .collect();
        assert_eq!(masked_order, vec![(1, 4, 9), (1, 5, 8)]);
    }

    #[test]
    fn transparent_sort_is_deterministic_on_depth_ties() {
        let mut collector = DrawCollector::default();
        collector.transparent.push(item(300, 20, 5, 99, 0.5));
        collector.transparent.push(item(100, 10, 3, 11, 0.5));
        collector.transparent.push(item(200, 10, 4, 11, 0.5));

        sort_collector(&mut collector);

        let model_order: Vec<u32> = collector.transparent.iter().map(|it| it.model_id).collect();
        assert_eq!(model_order, vec![100, 200, 300]);
    }
}
