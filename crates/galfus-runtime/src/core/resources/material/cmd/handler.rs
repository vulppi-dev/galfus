use super::types::*;
use super::utils::{pack_pbr_material, pack_standard_material};
use crate::core::resources::{
    MATERIAL_DEFINITION_PBR_ID, MATERIAL_DEFINITION_PBR_SLUG, MATERIAL_DEFINITION_STANDARD_ID,
    MATERIAL_DEFINITION_STANDARD_SLUG, MATERIAL_FALLBACK_ID, MaterialDefinitionRecord,
    MaterialInstanceRecord, MaterialShaderType, ShaderMaterialPreset, ShaderMaterialRecord,
};
use crate::core::state::EngineState;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

fn to_render_preset(preset: ShaderMaterialPreset) -> galfus_render::MaterialShaderBasePreset {
    match preset {
        ShaderMaterialPreset::Standard => galfus_render::MaterialShaderBasePreset::Standard,
        ShaderMaterialPreset::Pbr => galfus_render::MaterialShaderBasePreset::Pbr,
    }
}

fn to_render_shader_type(shader_type: MaterialShaderType) -> galfus_render::MaterialShaderType {
    match shader_type {
        MaterialShaderType::Model => galfus_render::MaterialShaderType::Model,
        MaterialShaderType::Particle => galfus_render::MaterialShaderType::Particle,
    }
}

fn definition_is_broken(definition: &MaterialDefinitionRecord) -> bool {
    definition.compile_error.is_some() || definition.compiled_shader_source.is_none()
}

fn compile_material_program(
    engine: &mut EngineState,
    preset: ShaderMaterialPreset,
    shader_type: MaterialShaderType,
    shader_source: String,
    shader_params_schema: HashMap<String, String>,
) -> Result<galfus_render::CompiledMaterialShader, String> {
    const MATERIAL_PROGRAM_CACHE_MAX_UNUSED_FRAMES: u64 = 600;
    const MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_SOFT: usize = 512;

    let spec = galfus_render::MaterialShaderCompileSpec {
        base_preset: to_render_preset(preset),
        shader_type: to_render_shader_type(shader_type),
        shader_source,
        shader_params_schema,
        capabilities: Default::default(),
    };

    let cache_key = {
        let mut hasher = DefaultHasher::new();
        to_render_preset(preset).hash(&mut hasher);
        to_render_shader_type(shader_type).hash(&mut hasher);
        spec.shader_source.hash(&mut hasher);
        let mut params: Vec<_> = spec.shader_params_schema.iter().collect();
        params.sort_by(|a, b| a.0.cmp(b.0));
        for (name, ty) in params {
            name.hash(&mut hasher);
            ty.hash(&mut hasher);
        }
        hasher.finish()
    };

    if let Some(compiled) = engine
        .universal_state
        .scene
        .material_program_cache
        .get(&cache_key)
        .cloned()
    {
        let frame_index = engine.runtime.frame_index();
        engine
            .universal_state
            .scene
            .material_program_cache_last_used_frame
            .insert(cache_key, frame_index);
        galfus_log::galfus_log_debug!(
            engine,
            "material.cache.hit",
            "key={} preset={:?} shader_type={:?}",
            cache_key,
            preset,
            shader_type
        );
        return Ok(compiled);
    }

    galfus_log::galfus_log_debug!(
        engine,
        "material.cache.miss",
        "key={} preset={:?} shader_type={:?}",
        cache_key,
        preset,
        shader_type
    );

    let compiled = galfus_render::compile_material_shader_spec(&spec)?;
    let frame_index = engine.runtime.frame_index();
    engine
        .universal_state
        .scene
        .material_program_cache
        .insert(cache_key, compiled.clone());
    engine
        .universal_state
        .scene
        .material_program_cache_last_used_frame
        .insert(cache_key, frame_index);
    let active_hashes: HashSet<u64> = engine
        .universal_state
        .scene
        .material_definitions
        .values()
        .filter_map(|record| {
            if record.compiled_shader_hash != 0 {
                Some(record.compiled_shader_hash)
            } else {
                None
            }
        })
        .collect();
    if engine.universal_state.scene.material_program_cache.len()
        > MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_SOFT
    {
        engine
            .universal_state
            .scene
            .material_program_cache
            .retain(|key, value| {
                let last_used = engine
                    .universal_state
                    .scene
                    .material_program_cache_last_used_frame
                    .get(key)
                    .copied()
                    .unwrap_or(frame_index);
                let within_window = frame_index.saturating_sub(last_used)
                    <= MATERIAL_PROGRAM_CACHE_MAX_UNUSED_FRAMES;
                within_window || active_hashes.contains(&value.hash)
            });
        engine
            .universal_state
            .scene
            .material_program_cache_last_used_frame
            .retain(|key, _| {
                engine
                    .universal_state
                    .scene
                    .material_program_cache
                    .contains_key(key)
            });
    }
    Ok(compiled)
}

fn builtin_shader_source(preset: ShaderMaterialPreset) -> &'static str {
    match preset {
        ShaderMaterialPreset::Standard => galfus_render::builtin_material_source(
            galfus_render::MaterialShaderBasePreset::Standard,
        ),
        ShaderMaterialPreset::Pbr => {
            galfus_render::builtin_material_source(galfus_render::MaterialShaderBasePreset::Pbr)
        }
    }
}

fn ensure_fallback_material_instance(engine: &mut EngineState) {
    let Some(standard_definition) = engine
        .universal_state
        .scene
        .material_definitions
        .get(&MATERIAL_DEFINITION_STANDARD_ID)
        .cloned()
    else {
        return;
    };
    if definition_is_broken(&standard_definition) {
        return;
    }
    let resources = &mut engine.universal_state.scene.realm3d;
    let mut fallback = resources
        .materials
        .remove(&MATERIAL_FALLBACK_ID)
        .unwrap_or_else(|| ShaderMaterialRecord::new_standard(Some("Fallback Material".into())));
    sync_material_from_definition(&mut fallback, &standard_definition);
    pack_standard_material(
        MATERIAL_FALLBACK_ID,
        &StandardOptions::default(),
        &mut fallback,
    );
    fallback.bind_group = None;
    resources.materials.insert(MATERIAL_FALLBACK_ID, fallback);

    engine.universal_state.scene.material_instances.insert(
        MATERIAL_FALLBACK_ID,
        MaterialInstanceRecord {
            material_id: MATERIAL_FALLBACK_ID,
            definition_id: MATERIAL_DEFINITION_STANDARD_ID,
            label: Some("fallback-standard".to_string()),
        },
    );
}

fn broken_definition_result(
    message: String,
    fallback_preserved: bool,
) -> CmdResultMaterialDefinition {
    let suffix = if fallback_preserved {
        " (fallback instance preserved)"
    } else {
        ""
    };
    CmdResultMaterialDefinition {
        success: false,
        message: format!("{message}{suffix}"),
    }
}

fn bootstrap_builtin_material_definitions(engine: &mut EngineState) {
    if engine
        .universal_state
        .scene
        .material_definitions
        .contains_key(&MATERIAL_DEFINITION_STANDARD_ID)
        && engine
            .universal_state
            .scene
            .material_definitions
            .contains_key(&MATERIAL_DEFINITION_PBR_ID)
    {
        return;
    }

    let candidates = [
        (
            MATERIAL_DEFINITION_STANDARD_ID,
            MATERIAL_DEFINITION_STANDARD_SLUG,
            ShaderMaterialPreset::Standard,
            Some("builtin-standard".to_string()),
        ),
        (
            MATERIAL_DEFINITION_PBR_ID,
            MATERIAL_DEFINITION_PBR_SLUG,
            ShaderMaterialPreset::Pbr,
            Some("builtin-pbr".to_string()),
        ),
    ];

    for (definition_id, slug, preset, label) in candidates {
        if engine
            .universal_state
            .scene
            .material_definitions
            .contains_key(&definition_id)
        {
            continue;
        }
        let compile_result = compile_material_program(
            engine,
            preset,
            MaterialShaderType::Model,
            builtin_shader_source(preset).to_string(),
            HashMap::new(),
        );
        let (compiled_source, compiled_hash, compile_error) = match compile_result {
            Ok(compiled) => (Some(compiled.source), compiled.hash, None),
            Err(error) => (None, 0, Some(error)),
        };
        engine.universal_state.scene.material_definitions.insert(
            definition_id,
            MaterialDefinitionRecord {
                definition_id,
                slug: slug.to_string(),
                label,
                base_preset: preset,
                shader_type: MaterialShaderType::Model,
                shader_source: Some(builtin_shader_source(preset).to_string()),
                shader_params_schema: HashMap::new(),
                compiled_shader_hash: compiled_hash,
                compiled_shader_source: compiled_source,
                compile_error,
            },
        );
    }
    ensure_fallback_material_instance(engine);
}

fn definition_by_slug(engine: &mut EngineState, slug: &str) -> Option<MaterialDefinitionRecord> {
    bootstrap_builtin_material_definitions(engine);
    engine
        .universal_state
        .scene
        .material_definitions
        .values()
        .find(|definition| definition.slug == slug)
        .cloned()
}

fn definition_by_id(
    engine: &mut EngineState,
    definition_id: u32,
) -> Option<MaterialDefinitionRecord> {
    bootstrap_builtin_material_definitions(engine);
    engine
        .universal_state
        .scene
        .material_definitions
        .get(&definition_id)
        .cloned()
}

fn sync_material_from_definition(
    record: &mut ShaderMaterialRecord,
    definition: &MaterialDefinitionRecord,
) {
    record.base_preset = definition.base_preset;
    record.preset = definition.base_preset;
    record.shader_type = definition.shader_type;
    record.shader_source = definition.shader_source.clone();
    record.shader_params_schema = definition.shader_params_schema.clone();
    record.compiled_shader_hash = definition.compiled_shader_hash;
    record.compiled_shader_source = definition.compiled_shader_source.clone();
    record.compile_error = definition.compile_error.clone();
}

pub fn engine_cmd_material_definition_create(
    engine: &mut EngineState,
    args: &CmdMaterialDefinitionCreateArgs,
) -> CmdResultMaterialDefinition {
    bootstrap_builtin_material_definitions(engine);

    if engine
        .universal_state
        .scene
        .material_definitions
        .contains_key(&args.definition_id)
    {
        return CmdResultMaterialDefinition {
            success: false,
            message: format!(
                "Material definition with id {} already exists",
                args.definition_id
            ),
        };
    }

    if definition_by_slug(engine, &args.slug).is_some() {
        return CmdResultMaterialDefinition {
            success: false,
            message: format!("Material definition slug '{}' already exists", args.slug),
        };
    }

    let shader_type = args.shader_type.unwrap_or(MaterialShaderType::Model);
    galfus_log::galfus_log_debug!(
        engine,
        "material.definition.compile.start",
        "definition={} slug={} preset={:?} shader_type={:?}",
        args.definition_id,
        args.slug,
        args.preset,
        shader_type
    );

    let compile_result = compile_material_program(
        engine,
        args.preset,
        shader_type,
        args.shader_source.clone(),
        args.shader_params_schema.clone().unwrap_or_default(),
    );

    let (compiled_source, compiled_hash, compile_error, compile_failed_msg) = match compile_result {
        Ok(compiled) => {
            galfus_log::galfus_log_debug!(
                engine,
                "material.definition.compile.ok",
                "definition={} slug={} hash={}",
                args.definition_id,
                args.slug,
                compiled.hash
            );
            (Some(compiled.source), compiled.hash, None, None)
        }
        Err(error) => {
            galfus_log::galfus_log_error!(
                engine,
                "material.definition.compile.fail",
                "definition={} slug={} error={}",
                args.definition_id,
                args.slug,
                error
            );
            let msg = format!(
                "Material definition '{}' compile failed: {}",
                args.slug, error
            );
            (None, 0, Some(error), Some(msg))
        }
    };

    engine.universal_state.scene.material_definitions.insert(
        args.definition_id,
        MaterialDefinitionRecord {
            definition_id: args.definition_id,
            slug: args.slug.clone(),
            label: args.label.clone(),
            base_preset: args.preset,
            shader_type,
            shader_source: Some(args.shader_source.clone()),
            shader_params_schema: args.shader_params_schema.clone().unwrap_or_default(),
            compiled_shader_hash: compiled_hash,
            compiled_shader_source: compiled_source,
            compile_error,
        },
    );
    ensure_fallback_material_instance(engine);

    if let Some(message) = compile_failed_msg {
        return broken_definition_result(message, true);
    }

    CmdResultMaterialDefinition {
        success: true,
        message: "Material definition created successfully".to_string(),
    }
}

pub fn engine_cmd_material_definition_update(
    engine: &mut EngineState,
    args: &CmdMaterialDefinitionUpdateArgs,
) -> CmdResultMaterialDefinition {
    bootstrap_builtin_material_definitions(engine);
    let Some(current) = definition_by_id(engine, args.definition_id) else {
        return CmdResultMaterialDefinition {
            success: false,
            message: format!(
                "Material definition with id {} not found",
                args.definition_id
            ),
        };
    };

    let slug = args.slug.clone().unwrap_or_else(|| current.slug.clone());
    if slug != current.slug {
        if definition_by_slug(engine, &slug).is_some() {
            return CmdResultMaterialDefinition {
                success: false,
                message: format!("Material definition slug '{}' already exists", slug),
            };
        }
    }

    let preset = args.preset.unwrap_or(current.base_preset);
    let shader_type = args.shader_type.unwrap_or(current.shader_type);
    let shader_source = args.shader_source.clone();
    let shader_params_schema = args
        .shader_params_schema
        .clone()
        .unwrap_or_else(|| current.shader_params_schema.clone());

    galfus_log::galfus_log_debug!(
        engine,
        "material.definition.compile.start",
        "definition={} slug={} preset={:?} shader_type={:?}",
        args.definition_id,
        slug,
        preset,
        shader_type
    );

    let compile_result = compile_material_program(
        engine,
        preset,
        shader_type,
        shader_source.clone(),
        shader_params_schema.clone(),
    );

    let (compiled_source, compiled_hash, compile_error, compile_failed_msg) = match compile_result {
        Ok(compiled) => {
            galfus_log::galfus_log_debug!(
                engine,
                "material.definition.compile.ok",
                "definition={} slug={} hash={}",
                args.definition_id,
                slug,
                compiled.hash
            );
            (Some(compiled.source), compiled.hash, None, None)
        }
        Err(error) => {
            galfus_log::galfus_log_error!(
                engine,
                "material.definition.compile.fail",
                "definition={} slug={} error={}",
                args.definition_id,
                slug,
                error
            );
            let msg = format!("Material definition '{}' compile failed: {}", slug, error);
            (None, 0, Some(error), Some(msg))
        }
    };

    if let Some(record) = engine
        .universal_state
        .scene
        .material_definitions
        .get_mut(&args.definition_id)
    {
        if let Some(label) = &args.label {
            record.label = Some(label.clone());
        }
        record.slug = slug;
        record.base_preset = preset;
        record.shader_type = shader_type;
        record.shader_source = Some(shader_source);
        record.shader_params_schema = shader_params_schema;
        record.compiled_shader_hash = compiled_hash;
        record.compiled_shader_source = compiled_source;
        record.compile_error = compile_error;
    }
    ensure_fallback_material_instance(engine);

    if let Some(message) = compile_failed_msg {
        return broken_definition_result(message, true);
    }

    CmdResultMaterialDefinition {
        success: true,
        message: "Material definition updated successfully".to_string(),
    }
}

pub fn engine_cmd_material_definition_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDefinitionDisposeArgs,
) -> CmdResultMaterialDefinition {
    if args.definition_id == MATERIAL_DEFINITION_STANDARD_ID
        || args.definition_id == MATERIAL_DEFINITION_PBR_ID
    {
        return CmdResultMaterialDefinition {
            success: false,
            message: "Builtin material definitions cannot be disposed".to_string(),
        };
    }

    if engine
        .universal_state
        .scene
        .material_definitions
        .remove(&args.definition_id)
        .is_some()
    {
        let active_hashes: HashSet<u64> = engine
            .universal_state
            .scene
            .material_definitions
            .values()
            .filter_map(|record| {
                if record.compiled_shader_hash != 0 {
                    Some(record.compiled_shader_hash)
                } else {
                    None
                }
            })
            .collect();
        engine
            .universal_state
            .scene
            .material_program_cache
            .retain(|_, value| active_hashes.contains(&value.hash));
        engine
            .universal_state
            .scene
            .material_program_cache_last_used_frame
            .retain(|key, _| {
                engine
                    .universal_state
                    .scene
                    .material_program_cache
                    .contains_key(key)
            });
        CmdResultMaterialDefinition {
            success: true,
            message: "Material definition disposed successfully".to_string(),
        }
    } else {
        CmdResultMaterialDefinition {
            success: false,
            message: format!(
                "Material definition with id {} not found",
                args.definition_id
            ),
        }
    }
}

pub fn engine_cmd_material_instance_create(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceCreateArgs,
) -> CmdResultMaterialInstance {
    let Some(definition) = definition_by_slug(engine, &args.slug) else {
        return CmdResultMaterialInstance {
            success: false,
            message: format!("Material definition slug '{}' not found", args.slug),
        };
    };
    if definition_is_broken(&definition) {
        return CmdResultMaterialInstance {
            success: false,
            message: format!(
                "Material definition slug '{}' is broken and cannot be instanced",
                args.slug
            ),
        };
    }

    let create_args = CmdMaterialCreateArgs {
        material_id: args.material_id,
        label: args.label.clone(),
        slug: args.slug.clone(),
        kind: MaterialKind::Shader,
        options: args.options.clone(),
    };

    let result = engine_cmd_material_create(engine, &create_args);
    if !result.success {
        return CmdResultMaterialInstance {
            success: false,
            message: result.message,
        };
    }

    engine.universal_state.scene.material_instances.insert(
        args.material_id,
        MaterialInstanceRecord {
            material_id: args.material_id,
            definition_id: definition.definition_id,
            label: args.label.clone(),
        },
    );

    CmdResultMaterialInstance {
        success: true,
        message: "Material instance created successfully".to_string(),
    }
}

pub fn engine_cmd_material_instance_update(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceUpdateArgs,
) -> CmdResultMaterialInstance {
    if let Some(slug) = args.slug.as_ref() {
        let Some(definition) = definition_by_slug(engine, slug) else {
            return CmdResultMaterialInstance {
                success: false,
                message: format!("Material definition slug '{}' not found", slug),
            };
        };
        if definition_is_broken(&definition) {
            return CmdResultMaterialInstance {
                success: false,
                message: format!(
                    "Material definition slug '{}' is broken and cannot be instanced",
                    slug
                ),
            };
        }
    }
    let update_args = CmdMaterialUpdateArgs {
        material_id: args.material_id,
        label: args.label.clone(),
        slug: args.slug.clone(),
        kind: Some(MaterialKind::Shader),
        options: args.options.clone(),
    };
    let result = engine_cmd_material_update(engine, &update_args);
    CmdResultMaterialInstance {
        success: result.success,
        message: result.message,
    }
}

pub fn engine_cmd_material_instance_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceDisposeArgs,
) -> CmdResultMaterialInstance {
    if engine
        .universal_state
        .scene
        .material_instances
        .remove(&args.material_id)
        .is_none()
    {
        return CmdResultMaterialInstance {
            success: false,
            message: format!("Material instance with id {} not found", args.material_id),
        };
    }

    let dispose_result = engine_cmd_material_dispose(
        engine,
        &CmdMaterialDisposeArgs {
            material_id: args.material_id,
        },
    );

    CmdResultMaterialInstance {
        success: dispose_result.success,
        message: dispose_result.message,
    }
}

pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
    bootstrap_builtin_material_definitions(engine);

    let Some(definition) = definition_by_slug(engine, &args.slug) else {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material definition slug '{}' not found", args.slug),
        };
    };
    if definition_is_broken(&definition) {
        return CmdResultMaterialCreate {
            success: false,
            message: format!(
                "Material definition slug '{}' is broken and cannot be used",
                args.slug
            ),
        };
    }

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

    let mut record = match definition.base_preset {
        ShaderMaterialPreset::Standard => ShaderMaterialRecord::new_standard(args.label.clone()),
        ShaderMaterialPreset::Pbr => ShaderMaterialRecord::new_pbr(args.label.clone()),
    };
    sync_material_from_definition(&mut record, &definition);

    match definition.base_preset {
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

    record.bind_group = None;
    {
        let resources = &mut engine.universal_state.scene.realm3d;
        resources.materials.insert(args.material_id, record);
    }

    engine.universal_state.scene.material_instances.insert(
        args.material_id,
        MaterialInstanceRecord {
            material_id: args.material_id,
            definition_id: definition.definition_id,
            label: args.label.clone(),
        },
    );

    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }

    CmdResultMaterialCreate {
        success: true,
        message: "Material created successfully".into(),
    }
}

pub fn engine_cmd_material_update(
    engine: &mut EngineState,
    args: &CmdMaterialUpdateArgs,
) -> CmdResultMaterialUpdate {
    let mut target_slug: Option<String> = args.slug.clone();

    let instance_definition_id = engine
        .universal_state
        .scene
        .material_instances
        .get(&args.material_id)
        .map(|instance| instance.definition_id);

    if target_slug.is_none() {
        if let Some(definition_id) = instance_definition_id {
            target_slug = engine
                .universal_state
                .scene
                .material_definitions
                .get(&definition_id)
                .map(|d| d.slug.clone());
        }
    }

    let Some(slug) = target_slug else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!(
                "Material {} has no instance slug; provide slug in update",
                args.material_id
            ),
        };
    };

    let Some(definition) = definition_by_slug(engine, &slug) else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!("Material definition slug '{}' not found", slug),
        };
    };
    if definition_is_broken(&definition) {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!(
                "Material definition slug '{}' is broken and cannot be used",
                slug
            ),
        };
    }

    let Some(record) = engine
        .universal_state
        .scene
        .realm3d
        .materials
        .get_mut(&args.material_id)
    else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        };
    };

    sync_material_from_definition(record, &definition);
    if let Some(label) = &args.label {
        record.label = Some(label.clone());
    }

    if let Some(opts) = &args.options {
        match opts {
            MaterialOptions::Standard(opts) => {
                pack_standard_material(args.material_id, opts, record)
            }
            MaterialOptions::Pbr(opts) => pack_pbr_material(args.material_id, opts, record),
        }
    }
    record.mark_structural_dirty();

    engine.universal_state.scene.material_instances.insert(
        args.material_id,
        MaterialInstanceRecord {
            material_id: args.material_id,
            definition_id: definition.definition_id,
            label: args.label.clone(),
        },
    );

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
    if args.material_id == MATERIAL_FALLBACK_ID {
        return CmdResultMaterialDispose {
            success: false,
            message: "Fallback material cannot be disposed".into(),
        };
    }

    engine
        .universal_state
        .scene
        .material_instances
        .remove(&args.material_id);

    if engine
        .universal_state
        .scene
        .realm3d
        .materials
        .remove(&args.material_id)
        .is_some()
    {
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
