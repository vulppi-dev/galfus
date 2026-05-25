use crate::core::resources::{
    SHADER_MATERIAL_INPUTS_PER_MATERIAL, ShaderMaterialRecord,
};
use glam::Vec4;

pub(crate) fn pack_schema_material(
    material_id: u32,
    params: &std::collections::HashMap<String, Vec4>,
    record: &mut ShaderMaterialRecord,
) {
    if record.inputs.len() != SHADER_MATERIAL_INPUTS_PER_MATERIAL as usize {
        record.inputs = vec![Vec4::ZERO; SHADER_MATERIAL_INPUTS_PER_MATERIAL as usize];
    }

    // Always keep per-material offset coherent when using schema path.
    let inputs_offset = material_id.saturating_mul(SHADER_MATERIAL_INPUTS_PER_MATERIAL);
    record.data_standard.inputs_offset_count =
        glam::UVec2::new(inputs_offset, SHADER_MATERIAL_INPUTS_PER_MATERIAL);
    record.data_pbr.inputs_offset_count =
        glam::UVec2::new(inputs_offset, SHADER_MATERIAL_INPUTS_PER_MATERIAL);

    for (name, value) in params {
        match name.as_str() {
            "baseColor" => record.inputs[0] = *value,
            "specColor" => record.inputs[1] = *value,
            "emissiveColor" => record.inputs[3] = *value,
            "toonParams" => {
                if record.inputs.len() > 4 {
                    record.inputs[4] = *value;
                }
            }
            "specPower" => record.inputs[2].x = value.x,
            "metallic" => record.inputs[2].x = value.x,
            "roughness" => record.inputs[2].y = value.x,
            "ao" => record.inputs[2].z = value.x,
            "normalScale" => record.inputs[3].x = value.x,
            _ => {
                if let Some(index_str) = name.strip_prefix("input")
                    && let Ok(index) = index_str.parse::<usize>()
                    && index < record.inputs.len()
                {
                    record.inputs[index] = *value;
                }
            }
        }
    }
}
