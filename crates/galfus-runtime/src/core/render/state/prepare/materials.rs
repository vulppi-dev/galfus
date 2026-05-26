use super::super::RenderState;
use crate::core::resources::{
    Material3dParams, SHADER_MATERIAL_INVALID_SLOT, SHADER_MATERIAL_TEXTURE_SLOTS,
    TEX_SOURCE_ATLAS, TEX_SOURCE_INVALID, TEX_SOURCE_STANDALONE,
};

fn set_uvec4_lane(vecs: &mut [glam::UVec4; 2], index: usize, value: u32) {
    let vec_index = index / 4;
    let lane = index % 4;
    let mut v = vecs[vec_index];
    match lane {
        0 => v.x = value,
        1 => v.y = value,
        2 => v.z = value,
        _ => v.w = value,
    }
    vecs[vec_index] = v;
}

fn pbr_to_material3d(params: &crate::core::resources::MaterialPbrParams) -> Material3dParams {
    Material3dParams {
        input_indices: params.input_indices,
        inputs_offset_count: params.inputs_offset_count,
        surface_flags: params.surface_flags,
        texture_slots: params.texture_slots,
        sampler_indices: params.sampler_indices,
        tex_sources: params.tex_sources,
        atlas_layers: params.atlas_layers,
        atlas_scale_bias: params.atlas_scale_bias,
    }
}

#[cfg(test)]
mod tests {
    use super::pbr_to_material3d;

    #[test]
    fn pbr_payload_maps_losslessly_to_material3d_payload() {
        let mut pbr = crate::core::resources::MaterialPbrParams::default();
        pbr.input_indices = glam::UVec4::new(3, 2, 1, 0);
        pbr.inputs_offset_count = glam::UVec2::new(42, 8);
        pbr.surface_flags = glam::UVec2::new(2, 17);
        pbr.texture_slots = [glam::UVec4::new(1, 2, 3, 4), glam::UVec4::new(5, 6, 7, 8)];
        pbr.sampler_indices = [glam::UVec4::new(2, 2, 2, 2), glam::UVec4::new(3, 3, 3, 3)];
        pbr.tex_sources = [glam::UVec4::new(0, 1, 2, 0), glam::UVec4::new(1, 2, 0, 1)];
        pbr.atlas_layers = [glam::UVec4::new(9, 8, 7, 6), glam::UVec4::new(5, 4, 3, 2)];
        pbr.atlas_scale_bias[0] = glam::Vec4::new(0.5, 0.5, 0.1, 0.2);
        pbr.atlas_scale_bias[7] = glam::Vec4::new(1.2, 1.3, 0.4, 0.5);

        let mapped = pbr_to_material3d(&pbr);

        assert_eq!(mapped.input_indices, pbr.input_indices);
        assert_eq!(mapped.inputs_offset_count, pbr.inputs_offset_count);
        assert_eq!(mapped.surface_flags, pbr.surface_flags);
        assert_eq!(mapped.texture_slots, pbr.texture_slots);
        assert_eq!(mapped.sampler_indices, pbr.sampler_indices);
        assert_eq!(mapped.tex_sources, pbr.tex_sources);
        assert_eq!(mapped.atlas_layers, pbr.atlas_layers);
        assert_eq!(mapped.atlas_scale_bias, pbr.atlas_scale_bias);
    }
}

impl RenderState {
    pub(crate) fn prepare_materials(&mut self, device: &wgpu::Device) {
        let bindings = self.bindings.as_mut().expect("bindings must exist");
        let library = self.library.as_ref().expect("library must exist");

        let mut material_ids: Vec<u32> = self.scene.materials.keys().copied().collect();
        material_ids.sort_unstable();

        let previous_slots = std::mem::take(&mut self.material_uniform_slots);
        self.material_uniform_slots = material_ids
            .iter()
            .copied()
            .enumerate()
            .map(|(slot, id)| (id, slot as u32))
            .collect();

        for id in material_ids {
            let Some(record) = self.scene.materials.get_mut(&id) else {
                continue;
            };
            let Some(slot) = self.material_uniform_slots.get(&id).copied() else {
                continue;
            };
            let slot_changed = previous_slots.get(&id).copied() != Some(slot);
            let mut atlas_changed = false;
            for slot in 0..SHADER_MATERIAL_TEXTURE_SLOTS {
                let tex_id = record.texture_ids[slot];
                let mut desired_source = TEX_SOURCE_INVALID;
                let mut desired_layer = 0u32;
                let mut desired_scale_bias = glam::Vec4::new(1.0, 1.0, 0.0, 0.0);

                if tex_id != SHADER_MATERIAL_INVALID_SLOT {
                    if let Some(entry) = self.scene.forward_atlas_entries.get(&tex_id) {
                        desired_source = TEX_SOURCE_ATLAS;
                        desired_layer = entry.layer;
                        desired_scale_bias = entry.uv_scale_bias;
                    } else {
                        desired_source = TEX_SOURCE_STANDALONE;
                        if self.external_textures.contains_key(&tex_id) {
                            desired_scale_bias = glam::Vec4::new(-1.0, -1.0, 1.0, 1.0);
                        }
                    }
                }

                let (tex_sources, atlas_layers, atlas_scale_bias) = match record.preset {
                    crate::core::resources::ShaderMaterialPreset::Standard => (
                        &mut record.data_standard.tex_sources,
                        &mut record.data_standard.atlas_layers,
                        &mut record.data_standard.atlas_scale_bias,
                    ),
                    crate::core::resources::ShaderMaterialPreset::Pbr => (
                        &mut record.data_pbr.tex_sources,
                        &mut record.data_pbr.atlas_layers,
                        &mut record.data_pbr.atlas_scale_bias,
                    ),
                };
                let current_source = match (slot / 4, slot % 4) {
                    (0, 0) => tex_sources[0].x,
                    (0, 1) => tex_sources[0].y,
                    (0, 2) => tex_sources[0].z,
                    (0, 3) => tex_sources[0].w,
                    (1, 0) => tex_sources[1].x,
                    (1, 1) => tex_sources[1].y,
                    (1, 2) => tex_sources[1].z,
                    _ => tex_sources[1].w,
                };
                let current_layer = match (slot / 4, slot % 4) {
                    (0, 0) => atlas_layers[0].x,
                    (0, 1) => atlas_layers[0].y,
                    (0, 2) => atlas_layers[0].z,
                    (0, 3) => atlas_layers[0].w,
                    (1, 0) => atlas_layers[1].x,
                    (1, 1) => atlas_layers[1].y,
                    (1, 2) => atlas_layers[1].z,
                    _ => atlas_layers[1].w,
                };
                let current_scale_bias = atlas_scale_bias[slot];
                if current_source != desired_source {
                    set_uvec4_lane(tex_sources, slot, desired_source);
                    atlas_changed = true;
                }
                if current_layer != desired_layer {
                    set_uvec4_lane(atlas_layers, slot, desired_layer);
                    atlas_changed = true;
                }
                if current_scale_bias != desired_scale_bias {
                    atlas_scale_bias[slot] = desired_scale_bias;
                    atlas_changed = true;
                }
            }

            if record.is_dirty || atlas_changed || slot_changed {
                let material_params = match record.preset {
                    crate::core::resources::ShaderMaterialPreset::Standard => record.data_standard,
                    crate::core::resources::ShaderMaterialPreset::Pbr => {
                        pbr_to_material3d(&record.data_pbr)
                    }
                };
                bindings.material_3d_pool.write(slot, &material_params);
                if record.is_dirty {
                    let inputs_offset = match record.preset {
                        crate::core::resources::ShaderMaterialPreset::Standard => {
                            record.data_standard.inputs_offset_count.x
                        }
                        crate::core::resources::ShaderMaterialPreset::Pbr => {
                            record.data_pbr.inputs_offset_count.x
                        }
                    };
                    bindings
                        .material_3d_inputs
                        .write_slice(inputs_offset, &record.inputs);
                }
                record.clear_dirty();
            }

            if record.bind_group.is_none() {
                let mut entries = Vec::with_capacity(12);
                entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.instance_pool.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.material_3d_pool.buffer(),
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(
                                std::mem::size_of::<Material3dParams>() as u64
                            )
                            .expect("nz"),
                        ),
                    }),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.material_3d_inputs.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });

                for slot in 0..SHADER_MATERIAL_TEXTURE_SLOTS {
                    let tex_id = record.texture_ids[slot];
                    let view = if tex_id != SHADER_MATERIAL_INVALID_SLOT {
                        self.scene
                            .textures
                            .get(&tex_id)
                            .map(|t| &t.view)
                            .or_else(|| self.external_textures.get(&tex_id))
                            .unwrap_or(&library.fallback_view)
                    } else {
                        &library.fallback_view
                    };
                    entries.push(wgpu::BindGroupEntry {
                        binding: (3 + slot) as u32,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }
                entries.push(wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.bones_pool.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });

                record.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup ShaderMaterial"),
                    layout: &library.layout_object_3d_material,
                    entries: &entries,
                }));
            }
        }
    }
}
