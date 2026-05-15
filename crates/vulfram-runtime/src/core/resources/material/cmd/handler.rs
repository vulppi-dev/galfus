use super::types::*;
use super::utils::{pack_pbr_material, pack_standard_material};
use crate::core::resources::{MATERIAL_FALLBACK_ID, ShaderMaterialPreset, ShaderMaterialRecord};
use crate::core::state::EngineState;

pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
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

    let preset = args.preset.unwrap_or(ShaderMaterialPreset::Standard);
    let mut record = match preset {
        ShaderMaterialPreset::Standard => ShaderMaterialRecord::new_standard(args.label.clone()),
        ShaderMaterialPreset::Pbr => ShaderMaterialRecord::new_pbr(args.label.clone()),
    };
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
    };
    record.bind_group = None;
    resources.materials.insert(args.material_id, record);

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
    let resources = &mut engine.universal_state.scene.realm3d;

    let Some(record) = resources.materials.get_mut(&args.material_id) else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        };
    };
    if let Some(preset) = args.preset {
        record.preset = preset;
        record.bind_group = None;
        record.mark_dirty();
    }

    if let Some(label) = &args.label {
        record.label = Some(label.clone());
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
