use super::types::*;
use super::utils::mark_global_materials_dirty;
use crate::core::buffers::state::UploadType;
use crate::core::image::{ImageBuffer, ImagePixels};
use crate::core::resources::texture::{
    TextureAsyncEvent, TextureDecodeJob, TextureDecodeResult, TextureRecord,
};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use crate::core::target::TargetId;

pub fn engine_cmd_texture_create_from_buffer(
    engine: &mut EngineState,
    args: &CmdTextureCreateFromBufferArgs,
) -> CmdResultTextureCreateFromBuffer {
    let render_resources = &engine.universal_state.scene.render_resources;
    if render_resources.textures.contains_key(&args.texture_id)
        || render_resources
            .forward_atlas_entries
            .contains_key(&args.texture_id)
        || render_resources
            .target_texture_binds
            .contains_key(&args.texture_id)
        || engine.texture_async.is_pending(args.texture_id)
    {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!(
                "Texture with id {} already exists or pending",
                args.texture_id
            ),
            pending: false,
        };
    }

    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
                pending: false,
            };
        }
    };
    if buffer.upload_type != UploadType::ImageData {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!(
                "Invalid buffer type. Expected ImageData, got {:?}",
                buffer.upload_type
            ),
            pending: false,
        };
    }

    let job = TextureDecodeJob {
        texture_id: args.texture_id,
        label: args.label.clone(),
        srgb: args.srgb,
        mode: args.mode,
        atlas_options: args.atlas_options.clone(),
        bytes: buffer.data,
    };
    if let Err(message) = engine.texture_async.enqueue(job) {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message,
            pending: false,
        };
    }

    CmdResultTextureCreateFromBuffer {
        success: true,
        message: "Texture decode queued".into(),
        pending: true,
    }
}

fn create_texture_from_image(
    engine: &mut EngineState,
    args: &CmdTextureCreateFromBufferArgs,
    image: ImageBuffer,
) -> CmdResultTextureCreateFromBuffer {
    let render_resources = &mut engine.universal_state.scene.render_resources;
    if render_resources.textures.contains_key(&args.texture_id)
        || render_resources
            .forward_atlas_entries
            .contains_key(&args.texture_id)
        || render_resources
            .target_texture_binds
            .contains_key(&args.texture_id)
    {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
            pending: false,
        };
    }
    let Some(device) = engine.device.as_ref() else {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: "Device not initialized".into(),
            pending: false,
        };
    };
    let Some(queue) = engine.queue.as_ref() else {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: "Queue not initialized".into(),
            pending: false,
        };
    };

    if matches!(args.mode, TextureCreateMode::ForwardAtlas) {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: "ForwardAtlas mode is not available in global registry path".into(),
            pending: false,
        };
    }

    let (format, bytes_per_row, rows_per_image, pixel_data): (
        wgpu::TextureFormat,
        Option<u32>,
        Option<u32>,
        Vec<u8>,
    ) = match image.pixels {
        ImagePixels::Rgba8(data) => {
            let format = if args.srgb.unwrap_or(true) {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            };
            (format, Some(4 * image.width), Some(image.height), data)
        }
        ImagePixels::Rgba16F(data) => (
            wgpu::TextureFormat::Rgba16Float,
            Some(8 * image.width),
            Some(image.height),
            bytemuck::cast_slice::<u16, u8>(&data).to_vec(),
        ),
    };

    let size = wgpu::Extent3d {
        width: image.width,
        height: image.height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: args.label.as_deref().or(Some("Texture From Buffer")),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &pixel_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row,
            rows_per_image,
        },
        size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    render_resources.textures.insert(
        args.texture_id,
        TextureRecord {
            label: args.label.clone(),
            _texture: texture,
            view,
            format,
        },
    );
    mark_global_materials_dirty(&mut engine.universal_state.scene.realm3d, args.texture_id);
    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }

    CmdResultTextureCreateFromBuffer {
        success: true,
        message: "Texture created successfully".into(),
        pending: false,
    }
}

fn apply_decoded_texture_result(engine: &mut EngineState, result: TextureDecodeResult) -> bool {
    if engine.texture_async.was_canceled(result.texture_id) {
        engine
            .runtime
            .push_event(crate::core::cmd::EngineEvent::System(
                SystemEvent::TextureReady {
                    window_id: 0,
                    texture_id: result.texture_id,
                    success: false,
                    message: "Texture decode canceled".into(),
                },
            ));
        return true;
    }

    let args = CmdTextureCreateFromBufferArgs {
        texture_id: result.texture_id,
        label: result.label.clone(),
        buffer_id: 0,
        srgb: result.srgb,
        mode: result.mode,
        atlas_options: result.atlas_options.clone(),
    };
    let response = match result.image {
        Some(image) => create_texture_from_image(engine, &args, image),
        None => CmdResultTextureCreateFromBuffer {
            success: false,
            message: result.message,
            pending: false,
        },
    };

    engine
        .runtime
        .push_event(crate::core::cmd::EngineEvent::System(
            SystemEvent::TextureReady {
                window_id: 0,
                texture_id: args.texture_id,
                success: response.success,
                message: response.message,
            },
        ));
    true
}

pub fn process_async_texture_results(engine: &mut EngineState) {
    if !engine.pending_texture_decode_results.is_empty()
        && engine.device.is_some()
        && engine.queue.is_some()
    {
        let pending = std::mem::take(&mut engine.pending_texture_decode_results);
        for pending_result in pending {
            let _ = apply_decoded_texture_result(engine, pending_result);
        }
    }

    let results = engine.texture_async.drain_results();
    for result in results {
        match result {
            TextureAsyncEvent::Started {
                texture_id,
                total_bytes,
            } => {
                engine
                    .runtime
                    .push_event(crate::core::cmd::EngineEvent::System(
                        SystemEvent::TextureProcessingStarted {
                            window_id: 0,
                            texture_id,
                            total_bytes,
                        },
                    ));
            }
            TextureAsyncEvent::Progress {
                texture_id,
                processed_bytes,
                total_bytes,
            } => {
                engine
                    .runtime
                    .push_event(crate::core::cmd::EngineEvent::System(
                        SystemEvent::TextureProcessingProgress {
                            window_id: 0,
                            texture_id,
                            processed_bytes,
                            total_bytes,
                        },
                    ));
            }
            TextureAsyncEvent::Finished {
                texture_id,
                success,
                message,
                total_bytes,
            } => {
                engine
                    .runtime
                    .push_event(crate::core::cmd::EngineEvent::System(
                        SystemEvent::TextureProcessingFinished {
                            window_id: 0,
                            texture_id,
                            success,
                            message,
                            total_bytes,
                        },
                    ));
            }
            TextureAsyncEvent::Result(result) => {
                if engine.device.is_none() || engine.queue.is_none() {
                    engine.pending_texture_decode_results.push(result);
                    continue;
                }
                let _ = apply_decoded_texture_result(engine, result);
            }
        }
    }
}

pub fn engine_cmd_texture_create_solid_color(
    engine: &mut EngineState,
    args: &CmdTextureCreateSolidColorArgs,
) -> CmdResultTextureCreateSolidColor {
    let render_resources = &mut engine.universal_state.scene.render_resources;
    if render_resources.textures.contains_key(&args.texture_id)
        || render_resources
            .forward_atlas_entries
            .contains_key(&args.texture_id)
        || render_resources
            .target_texture_binds
            .contains_key(&args.texture_id)
    {
        return CmdResultTextureCreateSolidColor {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
        };
    }
    if matches!(args.mode, TextureCreateMode::ForwardAtlas) {
        return CmdResultTextureCreateSolidColor {
            success: false,
            message: "ForwardAtlas mode is not available in global registry path".into(),
        };
    }
    let Some(device) = engine.device.as_ref() else {
        return CmdResultTextureCreateSolidColor {
            success: false,
            message: "Device not initialized".into(),
        };
    };
    let Some(queue) = engine.queue.as_ref() else {
        return CmdResultTextureCreateSolidColor {
            success: false,
            message: "Queue not initialized".into(),
        };
    };
    let format = if args.srgb.unwrap_or(true) {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    };
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: args.label.as_deref().or(Some("Solid Color Texture")),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let r = (args.color.x.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (args.color.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (args.color.z.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (args.color.w.clamp(0.0, 1.0) * 255.0) as u8;
    let data = [r, g, b, a];
    queue.write_texture(
        texture.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    render_resources.textures.insert(
        args.texture_id,
        TextureRecord {
            label: args.label.clone(),
            _texture: texture,
            view,
            format,
        },
    );
    mark_global_materials_dirty(&mut engine.universal_state.scene.realm3d, args.texture_id);
    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }
    CmdResultTextureCreateSolidColor {
        success: true,
        message: "Texture created successfully".into(),
    }
}

pub fn engine_cmd_texture_dispose(
    engine: &mut EngineState,
    args: &CmdTextureDisposeArgs,
) -> CmdResultTextureDispose {
    engine.texture_async.cancel(args.texture_id);
    engine
        .pending_texture_decode_results
        .retain(|pending| pending.texture_id != args.texture_id);
    let render_resources = &mut engine.universal_state.scene.render_resources;
    let mut removed = false;
    removed |= render_resources.textures.remove(&args.texture_id).is_some();
    removed |= render_resources
        .forward_atlas_entries
        .remove(&args.texture_id)
        .is_some();
    removed |= render_resources
        .target_texture_binds
        .remove(&args.texture_id)
        .is_some();
    if removed {
        mark_global_materials_dirty(&mut engine.universal_state.scene.realm3d, args.texture_id);
        for window_state in engine.window.states.values_mut() {
            window_state.is_dirty = true;
        }
        CmdResultTextureDispose {
            success: true,
            message: "Texture disposed successfully".into(),
        }
    } else {
        CmdResultTextureDispose {
            success: false,
            message: format!("Texture with id {} not found", args.texture_id),
        }
    }
}

pub fn engine_cmd_texture_bind_target(
    engine: &mut EngineState,
    args: &CmdTextureBindTargetArgs,
) -> CmdResultTextureBindTarget {
    let render_resources = &mut engine.universal_state.scene.render_resources;
    if render_resources.textures.contains_key(&args.texture_id)
        || render_resources
            .forward_atlas_entries
            .contains_key(&args.texture_id)
        || engine.texture_async.is_pending(args.texture_id)
    {
        return CmdResultTextureBindTarget {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
        };
    }
    let target_id = TargetId(args.target_id);
    render_resources.target_texture_binds.insert(
        args.texture_id,
        crate::core::resources::TargetTextureBinding {
            target_id,
            label: args.label.clone(),
        },
    );
    mark_global_materials_dirty(&mut engine.universal_state.scene.realm3d, args.texture_id);
    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }
    CmdResultTextureBindTarget {
        success: true,
        message: "Texture bound to target".into(),
    }
}
