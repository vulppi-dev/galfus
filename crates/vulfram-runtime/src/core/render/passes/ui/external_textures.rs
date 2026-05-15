use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceTable};
use crate::core::resources::RenderTarget;
use crate::core::target::{TargetId, TargetKind, TargetTable};
use crate::core::ui::UiState;
use crate::core::ui::renderer::ExternalTextureInput;

pub fn collect_external_textures(
    _render_state: &crate::core::render::RenderState,
    ui_state: &mut UiState,
    targets: &TargetTable,
    _target_layers: &crate::core::target::TargetLayerTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> Vec<ExternalTextureInput> {
    let target_surfaces = resolve_external_target_surfaces(auto_links, targets, realm_id);
    let mut static_entries: Vec<(TargetId, SurfaceId)> = Vec::new();
    for (target_id, (surface_id, source_realm_id)) in &target_surfaces {
        let _ = source_realm_id;
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        match target.kind {
            TargetKind::Texture => static_entries.push((*target_id, *surface_id)),
            _ => {}
        }
    }
    static_entries.sort_by_key(|(target_id, _)| target_id.0);

    let static_signature =
        hash_external_texture_signature(&static_entries, surface_targets, realm_id);
    let mut static_inputs = if let Some(cached) = ui_state.external_input_cache.get(&realm_id)
        && cached.signature == static_signature
    {
        cached.inputs.clone()
    } else {
        let mut resolved = Vec::new();
        for (target_id, surface_id) in &static_entries {
            let Some(surface_state) = surfaces.entries.get(surface_id) else {
                continue;
            };
            let Some(surface_target) = surface_targets.get(surface_id) else {
                continue;
            };
            let size = surface_state.value.size;
            let size = [size.x.max(1), size.y.max(1)];
            resolved.push(ExternalTextureInput {
                id: target_id.0,
                view: surface_target.view.clone(),
                size,
                source_ptr: surface_target as *const RenderTarget as usize,
            });
        }
        ui_state.external_input_cache.insert(
            realm_id,
            crate::core::ui::state::UiExternalInputCache {
                signature: static_signature,
                inputs: resolved.clone(),
            },
        );
        resolved
    };

    let mut alive_target_ids = std::collections::HashSet::new();
    let mut inputs = Vec::with_capacity(static_inputs.len());
    for input in static_inputs.drain(..) {
        alive_target_ids.insert(input.id);
        ui_state.external_textures.insert(input.id, input.size);
        inputs.push(input);
    }
    ui_state
        .external_textures
        .retain(|target_id, _| alive_target_ids.contains(target_id));

    inputs
}

fn hash_external_texture_signature(
    static_entries: &[(TargetId, SurfaceId)],
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    realm_id.hash(&mut hasher);
    for (target_id, surface_id) in static_entries {
        target_id.hash(&mut hasher);
        surface_id.hash(&mut hasher);
        if let Some(surface_target) = surface_targets.get(surface_id) {
            (surface_target as *const RenderTarget as usize).hash(&mut hasher);
            let size = surface_target.texture.size();
            size.width.hash(&mut hasher);
            size.height.hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn resolve_external_target_surfaces(
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    targets: &TargetTable,
    realm_id: RealmId,
) -> HashMap<TargetId, (SurfaceId, u32)> {
    let mut target_surfaces: HashMap<TargetId, (SurfaceId, u32)> = HashMap::new();
    for ((link_realm, target_id), link) in auto_links {
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Texture {
            continue;
        }
        match target_surfaces.entry(*target_id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert((link.surface_id, *link_realm));
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if *link_realm == realm_id.0 {
                    entry.insert((link.surface_id, *link_realm));
                }
            }
        }
    }
    target_surfaces
}

#[cfg(test)]
#[path = "external_textures_tests.rs"]
mod tests;
