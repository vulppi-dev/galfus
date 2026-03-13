use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceTable};
use crate::core::render::RenderState;
use crate::core::resources::RenderTarget;
use crate::core::target::{TargetId, TargetKind, TargetLayerTable, TargetTable};
use crate::core::ui::UiState;
use crate::core::ui::renderer::ExternalTextureInput;

pub fn collect_external_textures(
    render_state: &RenderState,
    ui_state: &mut UiState,
    targets: &TargetTable,
    target_layers: &TargetLayerTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> Vec<ExternalTextureInput> {
    let target_surfaces = resolve_external_target_surfaces(auto_links, targets, realm_id);
    let mut static_entries: Vec<(TargetId, SurfaceId)> = Vec::new();
    let mut widget_entries: Vec<(TargetId, u32)> = Vec::new();
    for (target_id, (surface_id, source_realm_id)) in &target_surfaces {
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        match target.kind {
            TargetKind::Texture => static_entries.push((*target_id, *surface_id)),
            TargetKind::WidgetRealmViewport => widget_entries.push((*target_id, *source_realm_id)),
            _ => {}
        }
    }
    static_entries.sort_by_key(|(target_id, _)| target_id.0);
    widget_entries.sort_by_key(|(target_id, source_realm_id)| (target_id.0, *source_realm_id));

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
    let mut inputs = Vec::with_capacity(static_inputs.len() + widget_entries.len());
    for input in static_inputs.drain(..) {
        alive_target_ids.insert(input.id);
        ui_state.external_textures.insert(input.id, input.size);
        inputs.push(input);
    }
    for (target_id, source_realm_id) in widget_entries {
        let camera_id =
            resolve_widget_camera_id(render_state, target_layers, target_id, source_realm_id);
        if let Some(input) = camera_texture_input(render_state, target_id.0, camera_id) {
            alive_target_ids.insert(target_id.0);
            ui_state.external_textures.insert(target_id.0, input.size);
            inputs.push(input);
        }
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
        if target.kind != TargetKind::Texture && target.kind != TargetKind::WidgetRealmViewport {
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

fn camera_texture_input(
    render_state: &RenderState,
    target_id: u64,
    camera_id: Option<u32>,
) -> Option<ExternalTextureInput> {
    let camera_id = camera_id?;
    let camera = render_state.camera_record(camera_id)?;
    let camera_target = camera
        .render_target
        .as_ref()
        .or(camera.post_target.as_ref())?;
    let texture_size = camera_target.texture.size();
    let size = [texture_size.width.max(1), texture_size.height.max(1)];
    Some(ExternalTextureInput {
        id: target_id,
        view: camera_target.view.clone(),
        size,
        source_ptr: camera_target as *const RenderTarget as usize,
    })
}

fn resolve_widget_camera_id(
    render_state: &RenderState,
    target_layers: &TargetLayerTable,
    target_id: TargetId,
    source_realm_id: u32,
) -> Option<u32> {
    if let Some(camera_id) =
        target_layers
            .entries
            .iter()
            .find_map(|((layer_realm, layer_target), layer)| {
                if *layer_target == target_id && *layer_realm == source_realm_id {
                    return layer.camera_id;
                }
                None
            })
    {
        return Some(camera_id);
    }

    if let Some(camera_id) = target_layers
        .entries
        .iter()
        .find_map(|((_, layer_target), layer)| {
            if *layer_target == target_id {
                return layer.camera_id;
            }
            None
        })
    {
        return Some(camera_id);
    }

    render_state
        .camera_order
        .first()
        .copied()
        .or_else(|| render_state.scene.cameras.keys().min().copied())
}

#[cfg(test)]
mod tests {
    use super::resolve_external_target_surfaces;
    use crate::core::realm::{AutoLink, RealmId, SurfaceId};
    use crate::core::target::{TargetId, TargetKind, TargetState, TargetTable};
    use std::collections::HashMap;

    fn target(kind: TargetKind) -> TargetState {
        TargetState {
            kind,
            window_id: None,
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        }
    }

    #[test]
    fn prefers_link_from_current_realm_for_same_target() {
        let target_id = TargetId(42);
        let mut targets = TargetTable::default();
        targets
            .entries
            .insert(target_id, target(TargetKind::WidgetRealmViewport));
        let mut auto_links = HashMap::new();
        auto_links.insert(
            (1, target_id),
            AutoLink {
                surface_id: SurfaceId(10),
                connector_id: None,
                present_id: None,
            },
        );
        auto_links.insert(
            (7, target_id),
            AutoLink {
                surface_id: SurfaceId(20),
                connector_id: None,
                present_id: None,
            },
        );

        let resolved = resolve_external_target_surfaces(&auto_links, &targets, RealmId(7));
        assert_eq!(resolved.get(&target_id), Some(&(SurfaceId(20), 7)));
    }

    #[test]
    fn ignores_non_external_target_kinds() {
        let mut targets = TargetTable::default();
        let window_target = TargetId(1);
        let texture_target = TargetId(2);
        targets
            .entries
            .insert(window_target, target(TargetKind::Window));
        targets
            .entries
            .insert(texture_target, target(TargetKind::Texture));
        let mut auto_links = HashMap::new();
        auto_links.insert(
            (3, window_target),
            AutoLink {
                surface_id: SurfaceId(11),
                connector_id: None,
                present_id: None,
            },
        );
        auto_links.insert(
            (3, texture_target),
            AutoLink {
                surface_id: SurfaceId(12),
                connector_id: None,
                present_id: None,
            },
        );

        let resolved = resolve_external_target_surfaces(&auto_links, &targets, RealmId(3));
        assert!(!resolved.contains_key(&window_target));
        assert_eq!(resolved.get(&texture_target), Some(&(SurfaceId(12), 3)));
    }
}
