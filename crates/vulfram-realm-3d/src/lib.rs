pub const SUPPORTED_RENDER_PASSES: &[&str] = &[
    "shadow",
    "light-cull",
    "skybox",
    "forward",
    "outline",
    "ssao",
    "ssao-blur",
    "bloom",
    "post",
    "compose",
    "ui",
];

pub fn supports_render_pass(pass_id: &str) -> bool {
    SUPPORTED_RENDER_PASSES.contains(&pass_id)
}

pub fn graph_is_compatible<'a>(pass_ids: impl IntoIterator<Item = &'a str>) -> bool {
    pass_ids.into_iter().all(supports_render_pass)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureRecordMeta {
    pub id: u32,
    pub label: Option<String>,
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardAtlasEntryMeta {
    pub id: u32,
    pub label: Option<String>,
    pub layer: u32,
    pub uv_scale_bias: [f32; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetTextureBindingMeta {
    pub texture_id: u32,
    pub target_id: vulfram_realm_core::TargetId,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlan {
    pub stale_ids: Vec<u32>,
    pub replace_ids: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraProjectionPlan {
    pub preserve_runtime_projection: bool,
    pub reset_projection_size: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelRecordMeta {
    pub transform: [f32; 16],
    pub translation: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: [f32; 4],
    pub flags: [u32; 4],
    pub outline_color: [f32; 4],
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub layer_mask: u32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub cast_outline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRecordMeta {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub ground_color: [f32; 4],
    pub view: [f32; 16],
    pub projection: [f32; 16],
    pub view_projection: [f32; 16],
    pub intensity_range: [f32; 2],
    pub spot_inner_outer: [f32; 2],
    pub kind_flags: [u32; 2],
    pub layer_mask: u32,
    pub cast_shadow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRecordMeta {
    pub label: Option<String>,
    pub data_bytes: Vec<u8>,
    pub inputs_bytes: Vec<u8>,
    pub texture_ids: Vec<u32>,
    pub surface_type: u32,
    pub topology: u32,
    pub polygon_mode: u32,
}

pub fn hash_texture_records(records: &[TextureRecordMeta]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    records.len().hash(&mut hasher);
    for record in records {
        record.id.hash(&mut hasher);
        record.label.hash(&mut hasher);
        record.width.hash(&mut hasher);
        record.height.hash(&mut hasher);
        record.depth_or_array_layers.hash(&mut hasher);
        record.format.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn hash_forward_atlas_entries(entries: &[ForwardAtlasEntryMeta]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entries.len().hash(&mut hasher);
    for entry in entries {
        entry.id.hash(&mut hasher);
        entry.label.hash(&mut hasher);
        entry.layer.hash(&mut hasher);
        bytemuck::bytes_of(&entry.uv_scale_bias).hash(&mut hasher);
    }
    hasher.finish()
}

pub fn hash_target_texture_binds(binds: &[TargetTextureBindingMeta]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    binds.len().hash(&mut hasher);
    for bind in binds {
        bind.texture_id.hash(&mut hasher);
        bind.target_id.hash(&mut hasher);
        bind.label.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn plan_camera_projection_update(
    previous_kind_flags: [u32; 2],
    previous_near_far: [f32; 2],
    previous_ortho_scale: f32,
    next_kind_flags: [u32; 2],
    next_near_far: [f32; 2],
    next_ortho_scale: f32,
    previous_projection_size: [u32; 2],
) -> CameraProjectionPlan {
    let projection_params_changed = previous_kind_flags != next_kind_flags
        || previous_near_far != next_near_far
        || (previous_ortho_scale - next_ortho_scale).abs() > f32::EPSILON;
    let has_previous_projection =
        previous_projection_size[0] > 0 && previous_projection_size[1] > 0;

    CameraProjectionPlan {
        preserve_runtime_projection: has_previous_projection && !projection_params_changed,
        reset_projection_size: projection_params_changed,
    }
}

pub fn model_record_changed(current: &ModelRecordMeta, next: &ModelRecordMeta) -> bool {
    current != next
}

pub fn light_record_changed(current: &LightRecordMeta, next: &LightRecordMeta) -> bool {
    current != next
}

pub fn material_record_changed(current: &MaterialRecordMeta, next: &MaterialRecordMeta) -> bool {
    current != next
}

pub fn plan_texture_record_sync(
    current: &[TextureRecordMeta],
    next: &[TextureRecordMeta],
) -> SyncPlan {
    let current_by_id: std::collections::HashMap<_, _> =
        current.iter().map(|record| (record.id, record)).collect();
    let next_by_id: std::collections::HashMap<_, _> =
        next.iter().map(|record| (record.id, record)).collect();

    let mut stale_ids: Vec<u32> = current_by_id
        .keys()
        .filter(|id| !next_by_id.contains_key(id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next
        .iter()
        .filter(|record| match current_by_id.get(&record.id) {
            Some(current) => *current != *record,
            None => true,
        })
        .map(|record| record.id)
        .collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn plan_forward_atlas_sync(
    current: &[ForwardAtlasEntryMeta],
    next: &[ForwardAtlasEntryMeta],
) -> SyncPlan {
    let current_by_id: std::collections::HashMap<_, _> =
        current.iter().map(|entry| (entry.id, entry)).collect();
    let next_by_id: std::collections::HashMap<_, _> =
        next.iter().map(|entry| (entry.id, entry)).collect();

    let mut stale_ids: Vec<u32> = current_by_id
        .keys()
        .filter(|id| !next_by_id.contains_key(id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next
        .iter()
        .filter(|entry| match current_by_id.get(&entry.id) {
            Some(current) => *current != *entry,
            None => true,
        })
        .map(|entry| entry.id)
        .collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn plan_target_texture_bind_sync(
    current: &[TargetTextureBindingMeta],
    next: &[TargetTextureBindingMeta],
) -> SyncPlan {
    let current_by_id: std::collections::HashMap<_, _> =
        current.iter().map(|bind| (bind.texture_id, bind)).collect();
    let next_by_id: std::collections::HashMap<_, _> =
        next.iter().map(|bind| (bind.texture_id, bind)).collect();

    let mut stale_ids: Vec<u32> = current_by_id
        .keys()
        .filter(|id| !next_by_id.contains_key(id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next
        .iter()
        .filter(|bind| match current_by_id.get(&bind.texture_id) {
            Some(current) => *current != *bind,
            None => true,
        })
        .map(|bind| bind.texture_id)
        .collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn plan_geometry_registry_sync(current_ids: &[u32], next_ids: &[u32]) -> SyncPlan {
    let current_ids: std::collections::HashSet<u32> = current_ids.iter().copied().collect();
    let next_ids: std::collections::HashSet<u32> = next_ids.iter().copied().collect();

    let mut stale_ids: Vec<u32> = current_ids.difference(&next_ids).copied().collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next_ids.difference(&current_ids).copied().collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CameraProjectionPlan, ForwardAtlasEntryMeta, LightRecordMeta, MaterialRecordMeta,
        ModelRecordMeta, SyncPlan, TargetTextureBindingMeta, TextureRecordMeta,
        graph_is_compatible, hash_forward_atlas_entries, hash_target_texture_binds,
        hash_texture_records, light_record_changed, material_record_changed, model_record_changed,
        plan_camera_projection_update, plan_forward_atlas_sync, plan_geometry_registry_sync,
        plan_target_texture_bind_sync, plan_texture_record_sync, supports_render_pass,
    };

    #[test]
    fn threed_realm_accepts_full_pipeline_passes() {
        assert!(supports_render_pass("shadow"));
        assert!(supports_render_pass("ui"));
        assert!(!supports_render_pass("unknown"));
        assert!(graph_is_compatible(["shadow", "forward", "compose", "ui"]));
        assert!(!graph_is_compatible(["shadow", "unknown"]));
    }

    #[test]
    fn texture_hash_changes_with_metadata() {
        let a = vec![TextureRecordMeta {
            id: 1,
            label: Some("albedo".into()),
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
            format: "rgba16float".into(),
        }];
        let b = vec![TextureRecordMeta {
            width: 128,
            ..a[0].clone()
        }];
        assert_ne!(hash_texture_records(&a), hash_texture_records(&b));
    }

    #[test]
    fn atlas_hash_changes_with_uv_scale_bias() {
        let a = vec![ForwardAtlasEntryMeta {
            id: 1,
            label: Some("atlas".into()),
            layer: 0,
            uv_scale_bias: [1.0, 1.0, 0.0, 0.0],
        }];
        let b = vec![ForwardAtlasEntryMeta {
            uv_scale_bias: [0.5, 1.0, 0.0, 0.0],
            ..a[0].clone()
        }];
        assert_ne!(
            hash_forward_atlas_entries(&a),
            hash_forward_atlas_entries(&b)
        );
    }

    #[test]
    fn target_bind_hash_changes_with_target_id() {
        let a = vec![TargetTextureBindingMeta {
            texture_id: 5,
            target_id: vulfram_realm_core::TargetId(10),
            label: Some("bind".into()),
        }];
        let b = vec![TargetTextureBindingMeta {
            target_id: vulfram_realm_core::TargetId(11),
            ..a[0].clone()
        }];
        assert_ne!(hash_target_texture_binds(&a), hash_target_texture_binds(&b));
    }

    #[test]
    fn texture_sync_plan_marks_stale_and_changed_records() {
        let current = vec![
            TextureRecordMeta {
                id: 1,
                label: Some("a".into()),
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
                format: "rgba16float".into(),
            },
            TextureRecordMeta {
                id: 2,
                label: Some("b".into()),
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
                format: "rgba16float".into(),
            },
        ];
        let next = vec![
            TextureRecordMeta {
                id: 1,
                label: Some("a2".into()),
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
                format: "rgba16float".into(),
            },
            TextureRecordMeta {
                id: 3,
                label: Some("c".into()),
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
                format: "rgba16float".into(),
            },
        ];

        assert_eq!(
            plan_texture_record_sync(&current, &next),
            SyncPlan {
                stale_ids: vec![2],
                replace_ids: vec![1, 3],
            }
        );
    }

    #[test]
    fn atlas_sync_plan_marks_changed_entries() {
        let current = vec![ForwardAtlasEntryMeta {
            id: 1,
            label: Some("atlas".into()),
            layer: 0,
            uv_scale_bias: [1.0, 1.0, 0.0, 0.0],
        }];
        let next = vec![ForwardAtlasEntryMeta {
            id: 1,
            label: Some("atlas".into()),
            layer: 1,
            uv_scale_bias: [1.0, 1.0, 0.0, 0.0],
        }];

        assert_eq!(
            plan_forward_atlas_sync(&current, &next),
            SyncPlan {
                stale_ids: vec![],
                replace_ids: vec![1],
            }
        );
    }

    #[test]
    fn target_bind_sync_plan_marks_removed_and_changed_binds() {
        let current = vec![
            TargetTextureBindingMeta {
                texture_id: 5,
                target_id: vulfram_realm_core::TargetId(10),
                label: Some("bind".into()),
            },
            TargetTextureBindingMeta {
                texture_id: 7,
                target_id: vulfram_realm_core::TargetId(20),
                label: Some("old".into()),
            },
        ];
        let next = vec![TargetTextureBindingMeta {
            texture_id: 5,
            target_id: vulfram_realm_core::TargetId(11),
            label: Some("bind".into()),
        }];

        assert_eq!(
            plan_target_texture_bind_sync(&current, &next),
            SyncPlan {
                stale_ids: vec![7],
                replace_ids: vec![5],
            }
        );
    }

    #[test]
    fn geometry_registry_sync_plan_marks_missing_and_stale_ids() {
        assert_eq!(
            plan_geometry_registry_sync(&[1, 2, 3], &[2, 3, 4]),
            SyncPlan {
                stale_ids: vec![1],
                replace_ids: vec![4],
            }
        );
    }

    #[test]
    fn camera_projection_plan_detects_preserve_vs_reset() {
        assert_eq!(
            plan_camera_projection_update(
                [1, 0],
                [0.1, 100.0],
                1.0,
                [1, 0],
                [0.1, 100.0],
                1.0,
                [1280, 720],
            ),
            CameraProjectionPlan {
                preserve_runtime_projection: true,
                reset_projection_size: false,
            }
        );
        assert_eq!(
            plan_camera_projection_update(
                [1, 0],
                [0.1, 100.0],
                1.0,
                [2, 0],
                [0.1, 100.0],
                1.0,
                [1280, 720],
            ),
            CameraProjectionPlan {
                preserve_runtime_projection: false,
                reset_projection_size: true,
            }
        );
    }

    #[test]
    fn entity_and_material_change_detectors_compare_semantic_fields() {
        let model = ModelRecordMeta {
            transform: [0.0; 16],
            translation: [0.0; 4],
            rotation: [0.0; 4],
            scale: [1.0, 1.0, 1.0, 0.0],
            flags: [0; 4],
            outline_color: [1.0, 0.0, 0.0, 1.0],
            geometry_id: 1,
            material_id: Some(2),
            layer_mask: 3,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
        };
        let changed_model = ModelRecordMeta {
            geometry_id: 9,
            ..model.clone()
        };
        assert!(!model_record_changed(&model, &model));
        assert!(model_record_changed(&model, &changed_model));

        let light = LightRecordMeta {
            position: [0.0; 4],
            direction: [0.0; 4],
            color: [1.0; 4],
            ground_color: [0.0; 4],
            view: [0.0; 16],
            projection: [0.0; 16],
            view_projection: [0.0; 16],
            intensity_range: [1.0, 10.0],
            spot_inner_outer: [0.0, 0.0],
            kind_flags: [0, 0],
            layer_mask: 1,
            cast_shadow: false,
        };
        let changed_light = LightRecordMeta {
            cast_shadow: true,
            ..light.clone()
        };
        assert!(!light_record_changed(&light, &light));
        assert!(light_record_changed(&light, &changed_light));

        let material = MaterialRecordMeta {
            label: Some("mat".into()),
            data_bytes: vec![1, 2, 3],
            inputs_bytes: vec![4, 5, 6],
            texture_ids: vec![1, 2],
            surface_type: 0,
            topology: 2,
            polygon_mode: 0,
        };
        let changed_material = MaterialRecordMeta {
            texture_ids: vec![1, 3],
            ..material.clone()
        };
        assert!(!material_record_changed(&material, &material));
        assert!(material_record_changed(&material, &changed_material));
    }
}
