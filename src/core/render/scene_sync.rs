use super::RenderState;
use crate::core::realm::RealmId;
use std::hash::{Hash, Hasher};

pub(super) fn sync_scene_from_realm_and_universal_resources(
    render_state: &mut RenderState,
    universal: &crate::core::realm::UniversalState,
    realm_id: RealmId,
) {
    let previous_cameras = std::mem::take(&mut render_state.scene.cameras);
    render_state.detached_cameras.extend(previous_cameras);
    let live_camera_ids: std::collections::HashSet<u32> = universal
        .realm_entities
        .values()
        .flat_map(|entities| entities.cameras.keys().copied())
        .collect();
    render_state
        .detached_cameras
        .retain(|camera_id, _| live_camera_ids.contains(camera_id));
    let mut previous_models = std::mem::take(&mut render_state.scene.models);
    let mut previous_lights = std::mem::take(&mut render_state.scene.lights);
    render_state.scene.cameras.clear();
    render_state.scene.models.clear();
    render_state.scene.lights.clear();

    if let Some(entities) = universal.realm_entities.get(&realm_id) {
        for (camera_id, node) in &entities.cameras {
            let mut record = render_state
                .detached_cameras
                .remove(camera_id)
                .unwrap_or_else(|| node.to_render_record());
            let previous_data = record.data;
            let previous_projection_size = record.last_projection_size;
            let previous_ortho_scale = record.ortho_scale;
            let projection_params_changed = previous_data.kind_flags != node.data.kind_flags
                || previous_data.near_far != node.data.near_far
                || (previous_ortho_scale - node.ortho_scale).abs() > f32::EPSILON;
            record.label = node.label.clone();
            record.data = node.data;
            if previous_projection_size.x > 0
                && previous_projection_size.y > 0
                && !projection_params_changed
            {
                // Keep runtime projection/aspect from the resolved target size.
                record.data.projection = previous_data.projection;
                record.data.view_projection = record.data.projection * record.data.view;
            } else if projection_params_changed {
                // Force projection rebuild on this frame with the current target size.
                record.last_projection_size = glam::UVec2::ZERO;
            }
            record.layer_mask = node.layer_mask;
            record.order = node.order;
            record.ortho_scale = node.ortho_scale;
            record.view_position = node.view_position.clone();
            record.mark_dirty();
            render_state.scene.cameras.insert(*camera_id, record);
        }
        for (model_id, node) in &entities.models {
            if let Some(mut record) = previous_models.remove(model_id) {
                let data_changed = record.data.transform != node.data.transform
                    || record.data.translation != node.data.translation
                    || record.data.rotation != node.data.rotation
                    || record.data.scale != node.data.scale
                    || record.data.flags != node.data.flags
                    || record.data.outline_color != node.data.outline_color;
                let metadata_changed = record.geometry_id != node.geometry_id
                    || record.material_id != node.material_id
                    || record.layer_mask != node.layer_mask
                    || record.cast_shadow != node.cast_shadow
                    || record.receive_shadow != node.receive_shadow
                    || record.cast_outline != node.cast_outline;
                record.label = node.label.clone();
                record.data = node.data;
                record.geometry_id = node.geometry_id;
                record.material_id = node.material_id;
                record.layer_mask = node.layer_mask;
                record.cast_shadow = node.cast_shadow;
                record.receive_shadow = node.receive_shadow;
                record.cast_outline = node.cast_outline;
                if data_changed || metadata_changed {
                    record.mark_dirty();
                }
                render_state.scene.models.insert(*model_id, record);
            } else {
                render_state.scene.models.insert(*model_id, node.clone());
            }
        }
        for (light_id, node) in &entities.lights {
            if let Some(mut record) = previous_lights.remove(light_id) {
                let changed = record.data.position != node.data.position
                    || record.data.direction != node.data.direction
                    || record.data.color != node.data.color
                    || record.data.ground_color != node.data.ground_color
                    || record.data.view != node.data.view
                    || record.data.projection != node.data.projection
                    || record.data.view_projection != node.data.view_projection
                    || record.data.intensity_range != node.data.intensity_range
                    || record.data.spot_inner_outer != node.data.spot_inner_outer
                    || record.data.kind_flags != node.data.kind_flags
                    || record.layer_mask != node.layer_mask
                    || record.cast_shadow != node.cast_shadow;
                record.label = node.label.clone();
                record.data = node.data;
                record.layer_mask = node.layer_mask;
                record.cast_shadow = node.cast_shadow;
                if changed {
                    record.mark_dirty();
                }
                render_state.scene.lights.insert(*light_id, record);
            } else {
                render_state.scene.lights.insert(*light_id, node.clone());
            }
        }
    } else {
        previous_models.clear();
        previous_lights.clear();
    }

    let mut previous_materials_standard =
        std::mem::take(&mut render_state.scene.materials_standard);
    render_state.scene.materials_standard.clear();
    for (material_id, node) in &universal.universal_resources.materials_standard {
        if let Some(mut record) = previous_materials_standard.remove(material_id) {
            let changed = record.label != node.label
                || bytemuck::bytes_of(&record.data) != bytemuck::bytes_of(&node.data)
                || record.inputs != node.inputs
                || record.texture_ids != node.texture_ids
                || record.surface_type != node.surface_type;
            record.label = node.label.clone();
            record.data = node.data;
            record.inputs = node.inputs.clone();
            record.texture_ids = node.texture_ids;
            record.surface_type = node.surface_type;
            if changed {
                record.mark_dirty();
                record.bind_group = None;
            }
            render_state
                .scene
                .materials_standard
                .insert(*material_id, record);
        } else {
            render_state
                .scene
                .materials_standard
                .insert(*material_id, node.clone());
        }
    }
    let mut previous_materials_pbr = std::mem::take(&mut render_state.scene.materials_pbr);
    render_state.scene.materials_pbr.clear();
    for (material_id, node) in &universal.universal_resources.materials_pbr {
        if let Some(mut record) = previous_materials_pbr.remove(material_id) {
            let changed = record.label != node.label
                || bytemuck::bytes_of(&record.data) != bytemuck::bytes_of(&node.data)
                || record.inputs != node.inputs
                || record.texture_ids != node.texture_ids
                || record.surface_type != node.surface_type;
            record.label = node.label.clone();
            record.data = node.data;
            record.inputs = node.inputs.clone();
            record.texture_ids = node.texture_ids;
            record.surface_type = node.surface_type;
            if changed {
                record.mark_dirty();
                record.bind_group = None;
            }
            render_state
                .scene
                .materials_pbr
                .insert(*material_id, record);
        } else {
            render_state
                .scene
                .materials_pbr
                .insert(*material_id, node.clone());
        }
    }
    let textures_hash = hash_texture_records(&universal.universal_resources.textures);
    if render_state.textures_sync_hash != textures_hash {
        sync_texture_records(
            &mut render_state.scene.textures,
            &universal.universal_resources.textures,
        );
        render_state.textures_sync_hash = textures_hash;
    }
    let atlas_hash =
        hash_forward_atlas_entries(&universal.universal_resources.forward_atlas_entries);
    if render_state.atlas_sync_hash != atlas_hash {
        sync_forward_atlas_entries(
            &mut render_state.scene.forward_atlas_entries,
            &universal.universal_resources.forward_atlas_entries,
        );
        render_state.atlas_sync_hash = atlas_hash;
    }
    let binds_hash = hash_target_texture_binds(&universal.universal_resources.target_texture_binds);
    if render_state.target_binds_sync_hash != binds_hash {
        sync_target_texture_binds(
            &mut render_state.target_texture_binds,
            &universal.universal_resources.target_texture_binds,
        );
        render_state.target_binds_sync_hash = binds_hash;
    }
}

fn hash_texture_records(
    textures: &std::collections::HashMap<u32, crate::core::resources::TextureRecord>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    textures.len().hash(&mut hasher);
    for (id, record) in textures {
        id.hash(&mut hasher);
        record.label.hash(&mut hasher);
        let size = record._texture.size();
        size.width.hash(&mut hasher);
        size.height.hash(&mut hasher);
        size.depth_or_array_layers.hash(&mut hasher);
        record._texture.format().hash(&mut hasher);
        (record as *const crate::core::resources::TextureRecord as usize).hash(&mut hasher);
    }
    hasher.finish()
}

fn hash_forward_atlas_entries(
    entries: &std::collections::HashMap<u32, crate::core::resources::ForwardAtlasEntry>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entries.len().hash(&mut hasher);
    for (id, entry) in entries {
        id.hash(&mut hasher);
        entry.label.hash(&mut hasher);
        entry.layer.hash(&mut hasher);
        bytemuck::bytes_of(&entry.uv_scale_bias.to_array()).hash(&mut hasher);
    }
    hasher.finish()
}

fn hash_target_texture_binds(
    binds: &std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    binds.len().hash(&mut hasher);
    for (texture_id, bind) in binds {
        texture_id.hash(&mut hasher);
        bind.target_id.hash(&mut hasher);
        bind.label.hash(&mut hasher);
    }
    hasher.finish()
}

fn sync_texture_records(
    current: &mut std::collections::HashMap<u32, crate::core::resources::TextureRecord>,
    next: &std::collections::HashMap<u32, crate::core::resources::TextureRecord>,
) {
    current.retain(|id, _| next.contains_key(id));
    for (id, record) in next {
        let needs_replace = match current.get(id) {
            Some(existing) => {
                existing.label != record.label
                    || existing._texture.size() != record._texture.size()
                    || existing._texture.format() != record._texture.format()
            }
            None => true,
        };
        if needs_replace {
            current.insert(*id, record.clone());
        }
    }
}

fn sync_forward_atlas_entries(
    current: &mut std::collections::HashMap<u32, crate::core::resources::ForwardAtlasEntry>,
    next: &std::collections::HashMap<u32, crate::core::resources::ForwardAtlasEntry>,
) {
    current.retain(|id, _| next.contains_key(id));
    for (id, entry) in next {
        let needs_replace = match current.get(id) {
            Some(existing) => {
                existing.label != entry.label
                    || existing.layer != entry.layer
                    || existing.uv_scale_bias != entry.uv_scale_bias
            }
            None => true,
        };
        if needs_replace {
            current.insert(*id, entry.clone());
        }
    }
}

fn sync_target_texture_binds(
    current: &mut std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
    next: &std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
) {
    current.retain(|id, _| next.contains_key(id));
    for (id, bind) in next {
        let needs_replace = match current.get(id) {
            Some(existing) => existing.target_id != bind.target_id || existing.label != bind.label,
            None => true,
        };
        if needs_replace {
            current.insert(*id, bind.clone());
        }
    }
}

pub(super) fn sync_window_geometry_registry(
    render_state: &mut RenderState,
    geometries: &std::collections::HashMap<u32, crate::core::realm::UniversalGeometryRecord>,
) {
    let Some(vertex) = render_state.vertex.as_mut() else {
        return;
    };
    for (geometry_id, record) in geometries {
        if vertex.records().contains_key(geometry_id) {
            continue;
        }
        let _ = vertex.create_geometry(*geometry_id, record.label.clone(), record.entries.clone());
    }

    let stale_ids: Vec<u32> = vertex
        .records()
        .keys()
        .filter(|geometry_id| !geometries.contains_key(geometry_id))
        .copied()
        .collect();
    for geometry_id in stale_ids {
        let _ = vertex.destroy_geometry(geometry_id);
    }
}
