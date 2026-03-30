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

#[cfg(test)]
mod tests {
    use super::{
        ForwardAtlasEntryMeta, TargetTextureBindingMeta, TextureRecordMeta, graph_is_compatible,
        hash_forward_atlas_entries, hash_target_texture_binds, hash_texture_records,
        supports_render_pass,
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
}
