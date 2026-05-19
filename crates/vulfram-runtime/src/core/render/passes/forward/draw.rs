use super::branches;
use crate::core::resources::SurfaceType;

pub(crate) fn draw_batches(
    render_pass: &mut wgpu::RenderPass,
    scene: &crate::core::render::state::RenderScene,
    library: &crate::core::render::state::ResourceLibrary,
    collector: &crate::core::render::state::DrawCollector,
    bindings: &crate::core::render::state::BindingSystem,
    vertex_sys: &mut crate::core::resources::VertexAllocatorSystem,
    frame_index: u64,
    device: &wgpu::Device,
    cache: &mut crate::core::render::cache::RenderCache,
    sample_count: u32,
    material_shader_modules: &mut std::collections::HashMap<u64, wgpu::ShaderModule>,
    log_events: &mut Vec<vulfram_log::LogEvent>,
) {
    // 1. PBR Opaque
    draw_group(
        render_pass,
        &collector.pbr_opaque,
        SurfaceType::Opaque,
        true, // is_pbr
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
        sample_count,
        material_shader_modules,
        log_events,
    );

    // 2. PBR Masked
    draw_group(
        render_pass,
        &collector.pbr_masked,
        SurfaceType::Masked,
        true,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
        sample_count,
        material_shader_modules,
        log_events,
    );

    // 3. Standard Opaque
    draw_group(
        render_pass,
        &collector.standard_opaque,
        SurfaceType::Opaque,
        false, // is_pbr
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
        sample_count,
        material_shader_modules,
        log_events,
    );

    // 4. Standard Masked
    draw_group(
        render_pass,
        &collector.standard_masked,
        SurfaceType::Masked,
        false,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
        sample_count,
        material_shader_modules,
        log_events,
    );

    // 5. PBR Transparent
    draw_group(
        render_pass,
        &collector.pbr_transparent,
        SurfaceType::Transparent,
        true,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
        sample_count,
        material_shader_modules,
        log_events,
    );

    // 6. Standard Transparent
    draw_group(
        render_pass,
        &collector.standard_transparent,
        SurfaceType::Transparent,
        false,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
        sample_count,
        material_shader_modules,
        log_events,
    );
}

fn draw_group(
    render_pass: &mut wgpu::RenderPass,
    items: &[crate::core::render::state::DrawItem],
    surface_type: SurfaceType,
    is_pbr: bool,
    scene: &crate::core::render::state::RenderScene,
    bindings: &crate::core::render::state::BindingSystem,
    vertex_sys: &mut crate::core::resources::VertexAllocatorSystem,
    frame_index: u64,
    device: &wgpu::Device,
    cache: &mut crate::core::render::cache::RenderCache,
    library: &crate::core::render::state::ResourceLibrary,
    sample_count: u32,
    material_shader_modules: &mut std::collections::HashMap<u64, wgpu::ShaderModule>,
    log_events: &mut Vec<vulfram_log::LogEvent>,
) {
    if items.is_empty() {
        return;
    }

    let mut current_state = None;
    let mut i = 0;
    while i < items.len() {
        let batch_start = i;
        let item = &items[i];
        let mat_id = item.material_id;
        let geom_id = item.geometry_id;
        let topology = item.topology;
        let polygon_mode = item.polygon_mode;
        let render_side = item.render_side;
        let shader_hash = item.compiled_shader_hash;
        let Some(material_record) = scene.materials.get(&mat_id) else {
            i += 1;
            continue;
        };
        let Some(shader_source) = material_record.compiled_shader_source.as_ref() else {
            i += 1;
            continue;
        };
        let shader_id = if shader_hash == 0 {
            1
        } else {
            shader_hash
        };
        material_shader_modules
            .entry(shader_id)
            .or_insert_with(|| {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Material Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(shader_source.clone().into()),
                })
            });
        let Some(shader_module) = material_shader_modules.get(&shader_id) else {
            i += 1;
            continue;
        };

        if current_state != Some((shader_id, topology, polygon_mode, render_side)) {
            let pipeline = if is_pbr {
                branches::pbr::get_pipeline(
                    cache,
                    frame_index,
                    device,
                    library,
                    surface_type,
                    topology,
                    polygon_mode,
                    render_side,
                    sample_count,
                    shader_id,
                    shader_module,
                )
            } else {
                branches::standard::get_pipeline(
                    cache,
                    frame_index,
                    device,
                    library,
                    surface_type,
                    topology,
                    polygon_mode,
                    render_side,
                    sample_count,
                    shader_id,
                    shader_module,
                )
            };
            render_pass.set_pipeline(pipeline);
            current_state = Some((shader_id, topology, polygon_mode, render_side));
        }

        while i < items.len() && items[i].material_id == mat_id && items[i].geometry_id == geom_id {
            i += 1;
        }
        let batch_count = (i - batch_start) as u32;

        if is_pbr {
            if let Some(material) = scene.materials.get(&mat_id) {
                if let Some(group) = material.bind_group.as_ref() {
                    let material_offset = bindings.material_pbr_pool.get_offset(mat_id) as u32;
                    render_pass.set_bind_group(1, group, &[material_offset]);
                }
                vulfram_log::vulfram_log_debug!(
                    log_events,
                    "forward.material",
                    "material={} preset={:?} has_bind_group={}",
                    mat_id,
                    material.base_preset,
                    material.bind_group.is_some()
                );
            }
        } else {
            if let Some(material) = scene.materials.get(&mat_id) {
                if let Some(group) = material.bind_group.as_ref() {
                    let material_offset = bindings.material_standard_pool.get_offset(mat_id) as u32;
                    render_pass.set_bind_group(1, group, &[material_offset]);
                }
                vulfram_log::vulfram_log_debug!(
                    log_events,
                    "forward.material",
                    "material={} preset={:?} has_bind_group={}",
                    mat_id,
                    material.base_preset,
                    material.bind_group.is_some()
                );
            }
        }

        if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
            if vertex_sys.bind(render_pass, geom_id).is_ok() {
                let first_instance = items[batch_start].instance_idx;
                vulfram_log::vulfram_log_debug!(
                    log_events,
                    "forward.draw",
                    "geom={} idx_count={} instances={}..{}",
                    geom_id,
                    index_info.count,
                    first_instance,
                    first_instance + batch_count
                );
                render_pass.draw_indexed(
                    0..index_info.count,
                    0,
                    first_instance..(first_instance + batch_count),
                );
            }
        }
    }
}
