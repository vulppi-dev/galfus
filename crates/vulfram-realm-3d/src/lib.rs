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
        ForwardAtlasEntryMeta, SyncPlan, TargetTextureBindingMeta, TextureRecordMeta,
        graph_is_compatible, hash_forward_atlas_entries, hash_target_texture_binds,
        hash_texture_records, plan_forward_atlas_sync, plan_geometry_registry_sync,
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
}
