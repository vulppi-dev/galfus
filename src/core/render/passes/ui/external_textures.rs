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
    let signature = hash_external_texture_signature(
        render_state,
        targets,
        target_layers,
        auto_links,
        surface_targets,
        realm_id,
    );
    if let Some(cached) = ui_state.external_input_cache.get(&realm_id)
        && cached.signature == signature
    {
        return cached.inputs.clone();
    }

    let mut alive_target_ids = std::collections::HashSet::new();
    let mut target_surfaces: HashMap<TargetId, (SurfaceId, u32)> = HashMap::new();

    for ((link_realm, target_id), link) in auto_links.iter() {
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

    let mut inputs = Vec::new();

    for (target_id, (surface_id, source_realm_id)) in target_surfaces {
        if let Some(target_state) = targets.entries.get(&target_id) {
            if target_state.kind == TargetKind::WidgetRealmViewport {
                let camera_id = resolve_widget_camera_id(
                    render_state,
                    target_layers,
                    target_id,
                    source_realm_id,
                );
                if let Some(input) = camera_texture_input(render_state, target_id.0, camera_id) {
                    alive_target_ids.insert(target_id.0);
                    ui_state.external_textures.insert(target_id.0, input.size);
                    inputs.push(input);
                    continue;
                }
            }
        }

        let Some(surface_state) = surfaces.entries.get(&surface_id) else {
            continue;
        };
        let Some(surface_target) = surface_targets.get(&surface_id) else {
            continue;
        };
        let size = surface_state.value.size;
        let size = [size.x.max(1), size.y.max(1)];
        alive_target_ids.insert(target_id.0);
        ui_state.external_textures.insert(target_id.0, size);

        inputs.push(ExternalTextureInput {
            id: target_id.0,
            view: surface_target.view.clone(),
            size,
            source_ptr: surface_target as *const RenderTarget as usize,
        });
    }
    ui_state
        .external_textures
        .retain(|target_id, _| alive_target_ids.contains(target_id));
    ui_state.external_input_cache.insert(
        realm_id,
        crate::core::ui::state::UiExternalInputCache {
            signature,
            inputs: inputs.clone(),
        },
    );

    inputs
}

fn hash_external_texture_signature(
    render_state: &RenderState,
    targets: &TargetTable,
    target_layers: &TargetLayerTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    realm_id.hash(&mut hasher);
    for ((link_realm, target_id), link) in auto_links {
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Texture && target.kind != TargetKind::WidgetRealmViewport {
            continue;
        }
        link_realm.hash(&mut hasher);
        target_id.hash(&mut hasher);
        link.surface_id.hash(&mut hasher);
        target.kind.hash(&mut hasher);
        if let Some(surface_target) = surface_targets.get(&link.surface_id) {
            (surface_target as *const RenderTarget as usize).hash(&mut hasher);
            let size = surface_target.texture.size();
            size.width.hash(&mut hasher);
            size.height.hash(&mut hasher);
        }
    }
    for ((layer_realm, layer_target), layer) in &target_layers.entries {
        layer_realm.hash(&mut hasher);
        layer_target.hash(&mut hasher);
        layer.camera_id.hash(&mut hasher);
    }
    for camera_id in &render_state.camera_order {
        camera_id.hash(&mut hasher);
        if let Some(camera) = render_state.camera_record(*camera_id) {
            let ptr = camera
                .render_target
                .as_ref()
                .or(camera.post_target.as_ref())
                .map(|target| target as *const RenderTarget as usize)
                .unwrap_or_default();
            ptr.hash(&mut hasher);
        }
    }
    hasher.finish()
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
