use super::types::*;
use super::utils::{pack_pbr_material, pack_standard_material};
use crate::core::resources::{MATERIAL_FALLBACK_ID, ShaderMaterialPreset, ShaderMaterialRecord};
use crate::core::state::EngineState;
use std::hash::{DefaultHasher, Hash, Hasher};

fn compile_material_program(
    preset: ShaderMaterialPreset,
    shader_source: Option<String>,
    shader_params_schema: std::collections::HashMap<String, String>,
) -> Result<vulfram_render::CompiledMaterialShader, String> {
    let base_preset = match preset {
        ShaderMaterialPreset::Standard => vulfram_render::MaterialShaderBasePreset::Standard,
        ShaderMaterialPreset::Pbr => vulfram_render::MaterialShaderBasePreset::Pbr,
    };
    let spec = vulfram_render::MaterialShaderCompileSpec {
        base_preset,
        shader_source,
        shader_params_schema,
    };
    vulfram_render::compile_material_shader_spec(&spec)
}

fn default_compiled_source(preset: ShaderMaterialPreset) -> vulfram_render::CompiledMaterialShader {
    let source = match preset {
        ShaderMaterialPreset::Standard => include_str!(
            "../../../render/passes/forward/branches/forward_standard.wgsl"
        )
        .to_string(),
        ShaderMaterialPreset::Pbr => {
            include_str!("../../../render/passes/forward/branches/forward_pbr.wgsl").to_string()
        }
    };
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    vulfram_render::CompiledMaterialShader {
        source,
        hash: hasher.finish(),
    }
}

pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
    let preset = args.preset.unwrap_or(ShaderMaterialPreset::Standard);

    {
        let resources = &mut engine.universal_state.scene.realm3d;
        if resources.materials.contains_key(&args.material_id) {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Material with id {} already exists", args.material_id),
            };
        }
        if args.kind != MaterialKind::Shader {
            return CmdResultMaterialCreate {
                success: false,
                message: "Unsupported material kind".into(),
            };
        }
    }

    let mut record = match preset {
        ShaderMaterialPreset::Standard => ShaderMaterialRecord::new_standard(args.label.clone()),
        ShaderMaterialPreset::Pbr => ShaderMaterialRecord::new_pbr(args.label.clone()),
    };
    record.base_preset = preset;
    record.shader_source = args.shader_source.clone();
    record.shader_params_schema = args.shader_params_schema.clone().unwrap_or_default();

    match preset {
        ShaderMaterialPreset::Standard => {
            let opts = match &args.options {
                Some(MaterialOptions::Standard(opts)) => opts.clone(),
                None => StandardOptions::default(),
                _ => StandardOptions::default(),
            };
            pack_standard_material(args.material_id, &opts, &mut record);
        }
        ShaderMaterialPreset::Pbr => {
            let opts = match &args.options {
                Some(MaterialOptions::Pbr(opts)) => opts.clone(),
                None => PbrOptions::default(),
                _ => PbrOptions::default(),
            };
            pack_pbr_material(args.material_id, &opts, &mut record);
        }
    }

    vulfram_log::vulfram_log_debug!(
        engine,
        "material.compile.start",
        "material={} preset={:?}",
        args.material_id,
        preset
    );

    match compile_material_program(
        preset,
        record.shader_source.clone(),
        record.shader_params_schema.clone(),
    ) {
        Ok(compiled) => {
            record.compiled_shader_hash = compiled.hash;
            record.compiled_shader_source = Some(compiled.source);
            record.compile_error = None;
            vulfram_log::vulfram_log_debug!(
                engine,
                "material.compile.ok",
                "material={} preset={:?} hash={}",
                args.material_id,
                preset,
                record.compiled_shader_hash
            );
        }
        Err(error) => {
            let fallback = default_compiled_source(preset);
            record.compiled_shader_hash = fallback.hash;
            record.compiled_shader_source = Some(fallback.source);
            record.compile_error = Some(error.clone());
            crate::core::system::push_error_event(
                engine,
                "material",
                format!("Material {} compile failed: {}", args.material_id, error),
                None,
                Some("material-upsert".into()),
            );
            vulfram_log::vulfram_log_error!(
                engine,
                "material.compile.fail",
                "material={} preset={:?} error={}",
                args.material_id,
                preset,
                error
            );
        }
    }

    record.bind_group = None;
    {
        let resources = &mut engine.universal_state.scene.realm3d;
        resources.materials.insert(args.material_id, record);
    }

    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }
    vulfram_log::vulfram_log_debug!(
        engine,
        "realm3d.state",
        "material-created material={} kind={:?} preset={:?} label={:?}",
        args.material_id,
        args.kind,
        preset,
        args.label
    );

    CmdResultMaterialCreate {
        success: true,
        message: "Material created successfully".into(),
    }
}

pub fn engine_cmd_material_update(
    engine: &mut EngineState,
    args: &CmdMaterialUpdateArgs,
) -> CmdResultMaterialUpdate {
    let (preset, shader_source, shader_params_schema) = {
        let resources = &mut engine.universal_state.scene.realm3d;
        let Some(record) = resources.materials.get_mut(&args.material_id) else {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Material with id {} not found", args.material_id),
            };
        };

        if let Some(preset) = args.preset {
            record.preset = preset;
            record.base_preset = preset;
            record.bind_group = None;
            record.mark_dirty();
        }

        if let Some(label) = &args.label {
            record.label = Some(label.clone());
        }
        if args.shader_source.is_some() {
            record.shader_source = args.shader_source.clone();
            record.mark_structural_dirty();
        }
        if let Some(schema) = &args.shader_params_schema {
            record.shader_params_schema = schema.clone();
            record.mark_structural_dirty();
        }

        if let Some(opts) = &args.options {
            match opts {
                MaterialOptions::Standard(opts) => {
                    pack_standard_material(args.material_id, opts, record);
                    record.mark_dirty();
                }
                MaterialOptions::Pbr(opts) => {
                    pack_pbr_material(args.material_id, opts, record);
                    record.mark_dirty();
                }
            }
        }

        (
            record.base_preset,
            record.shader_source.clone(),
            record.shader_params_schema.clone(),
        )
    };

    vulfram_log::vulfram_log_debug!(
        engine,
        "material.compile.start",
        "material={} preset={:?}",
        args.material_id,
        preset
    );

    let compile_result = compile_material_program(preset, shader_source, shader_params_schema);

    let (compile_error_for_event, compiled_hash_for_log) = {
        let resources = &mut engine.universal_state.scene.realm3d;
        let Some(record) = resources.materials.get_mut(&args.material_id) else {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Material with id {} not found", args.material_id),
            };
        };

        match compile_result {
            Ok(compiled) => {
                record.compiled_shader_hash = compiled.hash;
                record.compiled_shader_source = Some(compiled.source);
                record.compile_error = None;
                (None, record.compiled_shader_hash)
            }
            Err(error) => {
                let fallback = default_compiled_source(preset);
                record.compiled_shader_hash = fallback.hash;
                record.compiled_shader_source = Some(fallback.source);
                record.compile_error = Some(error.clone());
                (Some(error), record.compiled_shader_hash)
            }
        }
    };

    if let Some(error) = compile_error_for_event {
        crate::core::system::push_error_event(
            engine,
            "material",
            format!("Material {} compile failed: {}", args.material_id, error),
            None,
            Some("material-upsert".into()),
        );
        vulfram_log::vulfram_log_error!(
            engine,
            "material.compile.fail",
            "material={} preset={:?} error={}",
            args.material_id,
            preset,
            error
        );
    } else {
        vulfram_log::vulfram_log_debug!(
            engine,
            "material.compile.ok",
            "material={} preset={:?} hash={}",
            args.material_id,
            preset,
            compiled_hash_for_log
        );
    }

    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }

    CmdResultMaterialUpdate {
        success: true,
        message: "Material updated successfully".into(),
    }
}

pub fn engine_cmd_material_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDisposeArgs,
) -> CmdResultMaterialDispose {
    let resources = &mut engine.universal_state.scene.realm3d;

    if args.material_id == MATERIAL_FALLBACK_ID {
        return CmdResultMaterialDispose {
            success: false,
            message: "Fallback material cannot be disposed".into(),
        };
    }

    if resources.materials.remove(&args.material_id).is_some() {
        for window_state in engine.window.states.values_mut() {
            window_state.is_dirty = true;
        }
        CmdResultMaterialDispose {
            success: true,
            message: "Material disposed successfully".into(),
        }
    } else {
        CmdResultMaterialDispose {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        }
    }
}
