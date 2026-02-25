use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceTable};
use crate::core::render::RenderState;
use crate::core::resources::RenderTarget;
use crate::core::system::{UiViewportClass, UiViewportCommand};
use crate::core::target::{TargetId, TargetKind, TargetLayerTable, TargetTable};
use crate::core::ui::UiState;
use crate::core::ui::events::UiEvent;
use crate::core::ui::render::{hash_shapes, render_realm_documents, sync_ui_images};
use crate::core::ui::renderer::ExternalTextureInput;
use crate::core::window::{CursorIcon, EngineWindowState, UserAttentionType};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum UiPlatformAction {
    SetCursorIcon {
        window_id: u32,
        icon: CursorIcon,
    },
    OpenUrl {
        window_id: u32,
        realm_id: u32,
        url: String,
        new_tab: bool,
    },
    ClipboardSetText {
        window_id: u32,
        realm_id: u32,
        text: String,
    },
    ClipboardRequestCopy {
        window_id: u32,
        realm_id: u32,
    },
    ClipboardRequestCut {
        window_id: u32,
        realm_id: u32,
    },
    ClipboardRequestPaste {
        window_id: u32,
        realm_id: u32,
    },
    RequestFocus {
        window_id: u32,
    },
    RequestAttention {
        window_id: u32,
        attention: Option<UserAttentionType>,
    },
    ScreenshotRequest {
        window_id: u32,
        realm_id: u32,
    },
    SetWindowTitle {
        window_id: u32,
        title: String,
    },
    SetWindowSize {
        window_id: u32,
        width: u32,
        height: u32,
    },
    SetWindowPosition {
        window_id: u32,
        x: i32,
        y: i32,
    },
    SetWindowResizable {
        window_id: u32,
        value: bool,
    },
    SetWindowDecorations {
        window_id: u32,
        value: bool,
    },
    SetWindowState {
        window_id: u32,
        state: EngineWindowState,
    },
    EmitViewportSync {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        parent_viewport_id: Option<u64>,
        class: UiViewportClass,
        title: Option<String>,
    },
    EmitViewportCommand {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        command: UiViewportCommand,
    },
    EmitViewportFallbackEmbedded {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        parent_viewport_id: Option<u64>,
    },
}

pub fn pass_ui(
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    realm_id: RealmId,
    window_id: u32,
    window_focused: bool,
    ui_events: &mut Vec<UiEvent>,
    targets: &TargetTable,
    target_layers: &TargetLayerTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
    time_seconds: f64,
) -> Vec<UiPlatformAction> {
    ui_state.ensure_realm(realm_id);
    let external_inputs = collect_external_textures(
        render_state,
        ui_state,
        targets,
        target_layers,
        surfaces,
        auto_links,
        surface_targets,
        realm_id,
    );
    let (context, pixels_per_point, input_events, modifiers) = {
        let Some(ui_realm) = ui_state.realm_mut(realm_id) else {
            return Vec::new();
        };
        ui_realm.last_frame_index = frame_index;
        (
            ui_realm.context.clone(),
            ui_realm.pixels_per_point,
            ui_realm.drain_events(),
            ui_realm.modifiers,
        )
    };

    let screen_size = egui::vec2(target_size.x as f32, target_size.y as f32);
    let mut input = egui::RawInput::default();
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);
    input.screen_rect = Some(screen_rect);
    input.focused = window_focused;
    if let Some(viewport) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
        viewport.native_pixels_per_point = Some(pixels_per_point);
        viewport.inner_rect = Some(screen_rect);
        viewport.outer_rect = Some(screen_rect);
        viewport.focused = Some(window_focused);
    }
    input.time = Some(time_seconds);
    input.events = input_events;
    input.modifiers = modifiers;
    sync_ui_images(&context, ui_state);
    let layout_start = Instant::now();
    let output = context.run(input, |ctx| {
        render_realm_documents(
            ctx,
            ui_state,
            realm_id,
            target_size,
            ui_events,
            time_seconds,
        );
    });
    let needs_repaint = context.has_requested_repaint();
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        realm.needs_repaint = needs_repaint;
    }
    let platform_actions = collect_platform_actions(&output, window_id, realm_id);
    let layout_ms = layout_start.elapsed().as_secs_f32() * 1000.0;

    let tess_start = Instant::now();
    let shapes_hash = hash_shapes(&output.shapes);
    let cached = ui_state
        .realm_mut(realm_id)
        .and_then(|realm| realm.tessellation_cache.as_ref())
        .filter(|cache| {
            cache.shapes_hash == shapes_hash
                && cache.pixels_per_point == output.pixels_per_point
                && output.textures_delta.set.is_empty()
        })
        .map(|cache| cache.clipped.clone());
    let clipped_primitives =
        cached.unwrap_or_else(|| context.tessellate(output.shapes, output.pixels_per_point));
    let tess_ms = tess_start.elapsed().as_secs_f32() * 1000.0;
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        realm.profile.layout_ms = layout_ms;
        realm.profile.tessellate_ms = tess_ms;
        realm.tessellation_cache = Some(crate::core::ui::state::UiTessellationCache {
            shapes_hash,
            pixels_per_point: output.pixels_per_point,
            clipped: clipped_primitives.clone(),
        });
    }

    let renderer = render_state
        .ui_renderers
        .entry(realm_id)
        .or_insert_with(|| crate::core::ui::UiRenderer::new(device, queue, target_format));
    renderer.update_textures(device, queue, &output.textures_delta);
    renderer.update_external_textures(device, &external_inputs);
    let render_stats = renderer.render(
        device,
        queue,
        encoder,
        target_view,
        target_format,
        target_size,
        output.pixels_per_point,
        &clipped_primitives,
    );
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        realm.profile.upload_ms = render_stats.upload_ms;
        realm.profile.draw_ms = render_stats.draw_ms;
    }

    let debug = ui_state.debug;
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        if debug.enabled && debug.show_profile {
            let painter = realm.context.debug_painter();
            let text = format!(
                "UI input: {:.2}ms\nUI layout: {:.2}ms\nUI tess: {:.2}ms\nUI upload: {:.2}ms\nUI draw: {:.2}ms",
                realm.profile.input_routing_ms,
                realm.profile.layout_ms,
                realm.profile.tessellate_ms,
                realm.profile.upload_ms,
                realm.profile.draw_ms
            );
            painter.text(
                egui::pos2(8.0, 8.0),
                egui::Align2::LEFT_TOP,
                text,
                egui::TextStyle::Monospace.resolve(&realm.context.style()),
                egui::Color32::from_rgba_premultiplied(255, 255, 255, 200),
            );
        }
    }
    platform_actions
}

fn collect_platform_actions(
    output: &egui::FullOutput,
    window_id: u32,
    realm_id: RealmId,
) -> Vec<UiPlatformAction> {
    let mut actions = Vec::new();
    if let Some(icon) = map_cursor_icon(output.platform_output.cursor_icon) {
        actions.push(UiPlatformAction::SetCursorIcon { window_id, icon });
    }
    if let Some(open_url) = output.platform_output.open_url.as_ref() {
        actions.push(UiPlatformAction::OpenUrl {
            window_id,
            realm_id: realm_id.0,
            url: open_url.url.clone(),
            new_tab: open_url.new_tab,
        });
    }
    if !output.platform_output.copied_text.is_empty() {
        actions.push(UiPlatformAction::ClipboardSetText {
            window_id,
            realm_id: realm_id.0,
            text: output.platform_output.copied_text.clone(),
        });
    }
    for (viewport_id, viewport) in &output.viewport_output {
        let viewport_id = viewport_id_key(*viewport_id);
        let parent_viewport_id = Some(viewport_id_key(viewport.parent));
        actions.push(UiPlatformAction::EmitViewportSync {
            window_id,
            realm_id: realm_id.0,
            viewport_id,
            parent_viewport_id,
            class: map_viewport_class(viewport.class),
            title: viewport.builder.title.clone(),
        });
        if !matches!(viewport.class, egui::ViewportClass::Root) {
            actions.push(UiPlatformAction::EmitViewportFallbackEmbedded {
                window_id,
                realm_id: realm_id.0,
                viewport_id,
                parent_viewport_id,
            });
        }
        for command in &viewport.commands {
            if !matches!(viewport.class, egui::ViewportClass::Root) {
                if let Some(command) = map_viewport_command(command) {
                    actions.push(UiPlatformAction::EmitViewportCommand {
                        window_id,
                        realm_id: realm_id.0,
                        viewport_id,
                        command,
                    });
                }
                continue;
            }
            match command {
                egui::ViewportCommand::Title(title) => {
                    actions.push(UiPlatformAction::SetWindowTitle {
                        window_id,
                        title: title.clone(),
                    });
                }
                egui::ViewportCommand::InnerSize(size) => {
                    actions.push(UiPlatformAction::SetWindowSize {
                        window_id,
                        width: size.x.max(1.0).round() as u32,
                        height: size.y.max(1.0).round() as u32,
                    });
                }
                egui::ViewportCommand::OuterPosition(pos) => {
                    actions.push(UiPlatformAction::SetWindowPosition {
                        window_id,
                        x: pos.x.round() as i32,
                        y: pos.y.round() as i32,
                    });
                }
                egui::ViewportCommand::Resizable(value) => {
                    actions.push(UiPlatformAction::SetWindowResizable {
                        window_id,
                        value: *value,
                    });
                }
                egui::ViewportCommand::Decorations(value) => {
                    actions.push(UiPlatformAction::SetWindowDecorations {
                        window_id,
                        value: *value,
                    });
                }
                egui::ViewportCommand::Fullscreen(value) => {
                    actions.push(UiPlatformAction::SetWindowState {
                        window_id,
                        state: if *value {
                            EngineWindowState::Fullscreen
                        } else {
                            EngineWindowState::Windowed
                        },
                    });
                }
                egui::ViewportCommand::Minimized(value) => {
                    actions.push(UiPlatformAction::SetWindowState {
                        window_id,
                        state: if *value {
                            EngineWindowState::Minimized
                        } else {
                            EngineWindowState::Windowed
                        },
                    });
                }
                egui::ViewportCommand::Maximized(value) => {
                    actions.push(UiPlatformAction::SetWindowState {
                        window_id,
                        state: if *value {
                            EngineWindowState::Maximized
                        } else {
                            EngineWindowState::Windowed
                        },
                    });
                }
                egui::ViewportCommand::RequestCopy => {
                    actions.push(UiPlatformAction::ClipboardRequestCopy {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                egui::ViewportCommand::RequestCut => {
                    actions.push(UiPlatformAction::ClipboardRequestCut {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                egui::ViewportCommand::RequestPaste => {
                    actions.push(UiPlatformAction::ClipboardRequestPaste {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                egui::ViewportCommand::Focus => {
                    actions.push(UiPlatformAction::RequestFocus { window_id });
                }
                egui::ViewportCommand::RequestUserAttention(attention) => {
                    actions.push(UiPlatformAction::RequestAttention {
                        window_id,
                        attention: map_attention_type(*attention),
                    });
                }
                egui::ViewportCommand::Screenshot => {
                    actions.push(UiPlatformAction::ScreenshotRequest {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                _ => {
                    if let Some(command) = map_viewport_command(command) {
                        actions.push(UiPlatformAction::EmitViewportCommand {
                            window_id,
                            realm_id: realm_id.0,
                            viewport_id,
                            command,
                        });
                    }
                }
            }
        }
    }
    actions
}

fn map_attention_type(attention: egui::UserAttentionType) -> Option<UserAttentionType> {
    match attention {
        egui::UserAttentionType::Critical => Some(UserAttentionType::Critical),
        egui::UserAttentionType::Informational => Some(UserAttentionType::Informational),
        egui::UserAttentionType::Reset => None,
    }
}

fn map_viewport_class(class: egui::ViewportClass) -> UiViewportClass {
    match class {
        egui::ViewportClass::Root => UiViewportClass::Root,
        egui::ViewportClass::Deferred => UiViewportClass::Deferred,
        egui::ViewportClass::Immediate => UiViewportClass::Immediate,
        egui::ViewportClass::Embedded => UiViewportClass::Embedded,
    }
}

fn viewport_id_key(viewport_id: egui::ViewportId) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    viewport_id.hash(&mut hasher);
    hasher.finish()
}

fn map_viewport_command(command: &egui::ViewportCommand) -> Option<UiViewportCommand> {
    match command {
        egui::ViewportCommand::Close => Some(UiViewportCommand::Close),
        egui::ViewportCommand::Title(title) => Some(UiViewportCommand::Title {
            title: title.clone(),
        }),
        egui::ViewportCommand::InnerSize(size) => Some(UiViewportCommand::InnerSize {
            width: size.x,
            height: size.y,
        }),
        egui::ViewportCommand::OuterPosition(pos) => {
            Some(UiViewportCommand::OuterPosition { x: pos.x, y: pos.y })
        }
        egui::ViewportCommand::Resizable(value) => {
            Some(UiViewportCommand::Resizable { value: *value })
        }
        egui::ViewportCommand::Decorations(value) => {
            Some(UiViewportCommand::Decorations { value: *value })
        }
        egui::ViewportCommand::Fullscreen(value) => {
            Some(UiViewportCommand::Fullscreen { value: *value })
        }
        egui::ViewportCommand::Minimized(value) => {
            Some(UiViewportCommand::Minimized { value: *value })
        }
        egui::ViewportCommand::Maximized(value) => {
            Some(UiViewportCommand::Maximized { value: *value })
        }
        egui::ViewportCommand::Focus => Some(UiViewportCommand::Focus),
        egui::ViewportCommand::Screenshot => Some(UiViewportCommand::Screenshot),
        egui::ViewportCommand::CursorVisible(value) => {
            Some(UiViewportCommand::CursorVisible { value: *value })
        }
        egui::ViewportCommand::CursorGrab(mode) => Some(UiViewportCommand::CursorGrab {
            mode: format!("{:?}", mode),
        }),
        egui::ViewportCommand::IMEAllowed(value) => {
            Some(UiViewportCommand::ImeAllowed { value: *value })
        }
        egui::ViewportCommand::IMERect(rect) => Some(UiViewportCommand::ImeRect {
            min_x: rect.min.x,
            min_y: rect.min.y,
            max_x: rect.max.x,
            max_y: rect.max.y,
        }),
        _ => None,
    }
}

fn map_cursor_icon(icon: egui::CursorIcon) -> Option<CursorIcon> {
    match icon {
        egui::CursorIcon::None => None,
        egui::CursorIcon::Default => Some(CursorIcon::Default),
        egui::CursorIcon::ContextMenu => Some(CursorIcon::ContextMenu),
        egui::CursorIcon::Help => Some(CursorIcon::Help),
        egui::CursorIcon::PointingHand => Some(CursorIcon::Pointer),
        egui::CursorIcon::Progress => Some(CursorIcon::Progress),
        egui::CursorIcon::Wait => Some(CursorIcon::Wait),
        egui::CursorIcon::Cell => Some(CursorIcon::Cell),
        egui::CursorIcon::Crosshair => Some(CursorIcon::Crosshair),
        egui::CursorIcon::Text => Some(CursorIcon::Text),
        egui::CursorIcon::VerticalText => Some(CursorIcon::VerticalText),
        egui::CursorIcon::Alias => Some(CursorIcon::Alias),
        egui::CursorIcon::Copy => Some(CursorIcon::Copy),
        egui::CursorIcon::Move => Some(CursorIcon::Move),
        egui::CursorIcon::NoDrop => Some(CursorIcon::NoDrop),
        egui::CursorIcon::NotAllowed => Some(CursorIcon::NotAllowed),
        egui::CursorIcon::Grab => Some(CursorIcon::Grab),
        egui::CursorIcon::Grabbing => Some(CursorIcon::Grabbing),
        egui::CursorIcon::AllScroll => Some(CursorIcon::AllScroll),
        egui::CursorIcon::ResizeHorizontal => Some(CursorIcon::EwResize),
        egui::CursorIcon::ResizeNeSw => Some(CursorIcon::NeswResize),
        egui::CursorIcon::ResizeNwSe => Some(CursorIcon::NwseResize),
        egui::CursorIcon::ResizeVertical => Some(CursorIcon::NsResize),
        egui::CursorIcon::ResizeEast => Some(CursorIcon::EResize),
        egui::CursorIcon::ResizeSouthEast => Some(CursorIcon::SeResize),
        egui::CursorIcon::ResizeSouth => Some(CursorIcon::SResize),
        egui::CursorIcon::ResizeSouthWest => Some(CursorIcon::SwResize),
        egui::CursorIcon::ResizeWest => Some(CursorIcon::WResize),
        egui::CursorIcon::ResizeNorthWest => Some(CursorIcon::NwResize),
        egui::CursorIcon::ResizeNorth => Some(CursorIcon::NResize),
        egui::CursorIcon::ResizeNorthEast => Some(CursorIcon::NeResize),
        egui::CursorIcon::ResizeColumn => Some(CursorIcon::ColResize),
        egui::CursorIcon::ResizeRow => Some(CursorIcon::RowResize),
        egui::CursorIcon::ZoomIn => Some(CursorIcon::ZoomIn),
        egui::CursorIcon::ZoomOut => Some(CursorIcon::ZoomOut),
    }
}

fn collect_external_textures(
    render_state: &RenderState,
    ui_state: &mut UiState,
    targets: &TargetTable,
    target_layers: &TargetLayerTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> Vec<ExternalTextureInput> {
    ui_state.external_textures.clear();
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
        ui_state.external_textures.insert(target_id.0, size);

        inputs.push(ExternalTextureInput {
            id: target_id.0,
            view: surface_target.view.clone(),
            size,
            source_ptr: surface_target as *const RenderTarget as usize,
        });
    }

    inputs
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
