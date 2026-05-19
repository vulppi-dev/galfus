#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_texture_png(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: glam::UVec2,
    format: wgpu::TextureFormat,
    path: &str,
) -> Result<(), String> {
    if size.x == 0 || size.y == 0 {
        return Err("capture target has zero size".to_string());
    }

    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = size.x.saturating_mul(bytes_per_pixel);
    let padded_bytes_per_row = unpadded_bytes_per_row
        .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .saturating_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer_size = padded_bytes_per_row as u64 * size.y as u64;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("DebugCapture.Readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("DebugCapture.Encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(size.y),
            },
        },
        wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let map_result = receiver
        .recv()
        .map_err(|_| "map_async channel closed".to_string())?;
    if map_result.is_err() {
        readback.unmap();
        return Err("failed to map readback buffer".to_string());
    }

    let mapped = slice.get_mapped_range();
    let mut rgba = vec![0u8; (size.x * size.y * bytes_per_pixel) as usize];
    for y in 0..size.y as usize {
        let src_offset = y * padded_bytes_per_row as usize;
        let dst_offset = y * unpadded_bytes_per_row as usize;
        let src_row = &mapped[src_offset..src_offset + unpadded_bytes_per_row as usize];
        rgba[dst_offset..dst_offset + unpadded_bytes_per_row as usize].copy_from_slice(src_row);
    }
    drop(mapped);
    readback.unmap();

    match format {
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
            for px in rgba.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {}
        _ => {
            return Err(format!(
                "unsupported capture format: {:?} (expected RGBA8/BGRA8)",
                format
            ));
        }
    }

    let img = image::RgbaImage::from_raw(size.x, size.y, rgba)
        .ok_or_else(|| "failed to build RGBA image from readback bytes".to_string())?;

    let parent = Path::new(path).parent();
    if let Some(dir) = parent
        && !dir.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dir)
            .map_err(|err| format!("failed creating capture directory: {err}"))?;
    }
    img.save(path)
        .map_err(|err| format!("failed to save PNG capture: {err}"))?;
    Ok(())
}
