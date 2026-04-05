use super::*;

pub(super) fn collect_live_ui_image_ids(ui_state: &UiState) -> HashSet<u32> {
    let mut ids: HashSet<u32> = HashSet::new();
    for document in ui_state.documents.values() {
        for entry in document.nodes.values() {
            match &entry.node.props {
                UiNodeProps::Image {
                    source: UiImageSource::UiImage(image_id),
                    ..
                }
                | UiNodeProps::ImageButton {
                    source: UiImageSource::UiImage(image_id),
                    ..
                } => {
                    ids.insert(*image_id);
                }
                _ => {}
            }
        }
    }
    ids
}
pub(super) fn image_buffer_to_color_image(record: &UiImageRecord) -> egui::ColorImage {
    let size = [record.image.width as usize, record.image.height as usize];
    let mut pixels = Vec::with_capacity(size[0] * size[1]);

    match &record.image.pixels {
        ImagePixels::Rgba8(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                pixels.push(egui::Color32::from_rgba_unmultiplied(
                    chunk[0], chunk[1], chunk[2], chunk[3],
                ));
            }
        }
        ImagePixels::Rgba16F(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                let r = half::f16::from_bits(chunk[0]).to_f32();
                let g = half::f16::from_bits(chunk[1]).to_f32();
                let b = half::f16::from_bits(chunk[2]).to_f32();
                let a = half::f16::from_bits(chunk[3]).to_f32();
                pixels.push(egui::Color32::from_rgba_unmultiplied(
                    to_u8(r),
                    to_u8(g),
                    to_u8(b),
                    to_u8(a),
                ));
            }
        }
    }

    egui::ColorImage { size, pixels }
}

fn to_u8(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0).round() as u8
}
