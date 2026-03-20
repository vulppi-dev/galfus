use super::types::{MaterialSampler, PbrOptions, StandardOptions};
use crate::core::resources::{
    MaterialPbrRecord, MaterialStandardRecord, PBR_INPUTS_PER_MATERIAL, PBR_TEXTURE_SLOTS,
    STANDARD_INPUTS_PER_MATERIAL, STANDARD_TEXTURE_SLOTS,
};
use glam::Vec4;

pub(crate) fn pack_standard_material(
    material_id: u32,
    opts: &StandardOptions,
    record: &mut MaterialStandardRecord,
) {
    let previous_texture_ids = record.texture_ids;
    let inputs_offset = material_id.saturating_mul(STANDARD_INPUTS_PER_MATERIAL);

    record.data.inputs_offset_count = glam::UVec2::new(inputs_offset, STANDARD_INPUTS_PER_MATERIAL);

    let mut flags = opts.flags.unwrap_or(record.data.surface_flags.y);
    if opts.spec_color.is_some() || opts.spec_power.is_some() || opts.spec_tex_id.is_some() {
        flags |= 1;
    }

    let surface_type = opts.surface_type.unwrap_or(record.surface_type);
    record.data.surface_flags = glam::UVec2::new(surface_type as u32, flags);

    let mut texture_slots = record.data.texture_slots;
    let mut sampler_indices = record.data.sampler_indices;
    let tex_sources = record.data.tex_sources;
    let atlas_layers = record.data.atlas_layers;
    let atlas_scale_bias = record.data.atlas_scale_bias;

    let assign_slot = |slots: &mut [glam::UVec4; 2], index: usize, value: u32| {
        let vec_index = index / 4;
        let lane = index % 4;
        let mut vec = slots[vec_index];
        match lane {
            0 => vec.x = value,
            1 => vec.y = value,
            2 => vec.z = value,
            _ => vec.w = value,
        }
        slots[vec_index] = vec;
    };

    let assign_sampler = |samplers: &mut [glam::UVec4; 2], index: usize, value: u32| {
        let vec_index = index / 4;
        let lane = index % 4;
        let mut vec = samplers[vec_index];
        match lane {
            0 => vec.x = value,
            1 => vec.y = value,
            2 => vec.z = value,
            _ => vec.w = value,
        }
        samplers[vec_index] = vec;
    };

    if let Some(tex_id) = opts.base_tex_id {
        let slot = 0;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 0, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                0,
                opts.base_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.base_sampler {
        assign_sampler(&mut sampler_indices, 0, sampler as u32);
    }

    if let Some(tex_id) = opts.spec_tex_id {
        let slot = 1;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 1, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                1,
                opts.spec_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.spec_sampler {
        assign_sampler(&mut sampler_indices, 1, sampler as u32);
    }

    if let Some(tex_id) = opts.normal_tex_id {
        let slot = 2;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 2, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                2,
                opts.normal_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.normal_sampler {
        assign_sampler(&mut sampler_indices, 2, sampler as u32);
    }

    if let Some(tex_id) = opts.toon_ramp_tex_id {
        let slot = 3;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 3, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                3,
                opts.toon_ramp_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.toon_ramp_sampler {
        assign_sampler(&mut sampler_indices, 3, sampler as u32);
    }

    if let Some(tex_id) = opts.emissive_tex_id {
        let slot = 4;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 4, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                4,
                opts.emissive_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.emissive_sampler {
        assign_sampler(&mut sampler_indices, 4, sampler as u32);
    }

    record.data.texture_slots = texture_slots;
    record.data.sampler_indices = sampler_indices;
    record.data.tex_sources = tex_sources;
    record.data.atlas_layers = atlas_layers;
    record.data.atlas_scale_bias = atlas_scale_bias;

    record.surface_type = surface_type;
    if let Some(topology) = opts.topology {
        record.topology = topology;
    }
    if let Some(polygon_mode) = opts.polygon_mode {
        record.polygon_mode = polygon_mode;
    }
    if record.texture_ids != previous_texture_ids {
        record.bind_group = None;
    }
    if record.inputs.len() != STANDARD_INPUTS_PER_MATERIAL as usize {
        record.inputs = vec![Vec4::ZERO; STANDARD_INPUTS_PER_MATERIAL as usize];
    }
    if let Some(color) = opts.base_color {
        record.inputs[0] = color;
    }
    if let Some(color) = opts.spec_color {
        record.inputs[1] = color;
    }
    if let Some(power) = opts.spec_power {
        record.inputs[2].x = power;
    }
    if let Some(color) = opts.emissive_color {
        record.inputs[3] = color;
    }
    if let Some(toon_params) = opts.toon_params {
        if record.inputs.len() > 4 {
            record.inputs[4] = toon_params;
        }
    }
}

pub(crate) fn pack_pbr_material(
    material_id: u32,
    opts: &PbrOptions,
    record: &mut MaterialPbrRecord,
) {
    let previous_texture_ids = record.texture_ids;
    let inputs_offset = material_id.saturating_mul(PBR_INPUTS_PER_MATERIAL);

    record.data.inputs_offset_count = glam::UVec2::new(inputs_offset, PBR_INPUTS_PER_MATERIAL);

    let surface_type = opts.surface_type.unwrap_or(record.surface_type);
    let flags = opts.flags.unwrap_or(record.data.surface_flags.y);
    record.data.surface_flags = glam::UVec2::new(surface_type as u32, flags);

    let mut texture_slots = record.data.texture_slots;
    let mut sampler_indices = record.data.sampler_indices;
    let tex_sources = record.data.tex_sources;
    let atlas_layers = record.data.atlas_layers;
    let atlas_scale_bias = record.data.atlas_scale_bias;

    let assign_slot = |slots: &mut [glam::UVec4; 2], index: usize, value: u32| {
        let vec_index = index / 4;
        let lane = index % 4;
        let mut vec = slots[vec_index];
        match lane {
            0 => vec.x = value,
            1 => vec.y = value,
            2 => vec.z = value,
            _ => vec.w = value,
        }
        slots[vec_index] = vec;
    };

    let assign_sampler = |samplers: &mut [glam::UVec4; 2], index: usize, value: u32| {
        let vec_index = index / 4;
        let lane = index % 4;
        let mut vec = samplers[vec_index];
        match lane {
            0 => vec.x = value,
            1 => vec.y = value,
            2 => vec.z = value,
            _ => vec.w = value,
        }
        samplers[vec_index] = vec;
    };

    if let Some(tex_id) = opts.base_tex_id {
        let slot = 0;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 0, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                0,
                opts.base_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.base_sampler {
        assign_sampler(&mut sampler_indices, 0, sampler as u32);
    }

    if let Some(tex_id) = opts.normal_tex_id {
        let slot = 1;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 1, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                1,
                opts.normal_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.normal_sampler {
        assign_sampler(&mut sampler_indices, 1, sampler as u32);
    }

    if let Some(tex_id) = opts.metallic_roughness_tex_id {
        let slot = 2;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 2, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                2,
                opts.metallic_roughness_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.metallic_roughness_sampler {
        assign_sampler(&mut sampler_indices, 2, sampler as u32);
    }

    if let Some(tex_id) = opts.emissive_tex_id {
        let slot = 3;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 3, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                3,
                opts.emissive_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.emissive_sampler {
        assign_sampler(&mut sampler_indices, 3, sampler as u32);
    }

    if let Some(tex_id) = opts.ao_tex_id {
        let slot = 4;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 4, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                4,
                opts.ao_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    } else if let Some(sampler) = opts.ao_sampler {
        assign_sampler(&mut sampler_indices, 4, sampler as u32);
    }

    record.data.texture_slots = texture_slots;
    record.data.sampler_indices = sampler_indices;
    record.data.tex_sources = tex_sources;
    record.data.atlas_layers = atlas_layers;
    record.data.atlas_scale_bias = atlas_scale_bias;

    record.surface_type = surface_type;
    if let Some(topology) = opts.topology {
        record.topology = topology;
    }
    if let Some(polygon_mode) = opts.polygon_mode {
        record.polygon_mode = polygon_mode;
    }
    if record.texture_ids != previous_texture_ids {
        record.bind_group = None;
    }
    if record.inputs.len() != PBR_INPUTS_PER_MATERIAL as usize {
        record.inputs = vec![Vec4::ZERO; PBR_INPUTS_PER_MATERIAL as usize];
    }
    if let Some(color) = opts.base_color {
        record.inputs[0] = color;
    }
    if let Some(color) = opts.emissive_color {
        record.inputs[1] = color;
    }

    let mut pbr_params = record.inputs[2];
    if let Some(metallic) = opts.metallic {
        pbr_params.x = metallic;
    }
    if let Some(roughness) = opts.roughness {
        pbr_params.y = roughness;
    }
    if let Some(ao) = opts.ao {
        pbr_params.z = ao;
    }
    record.inputs[2] = pbr_params;

    if let Some(normal_scale) = opts.normal_scale {
        record.inputs[3].x = normal_scale;
    }
}
