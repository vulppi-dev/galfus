use std::time::Instant;

#[derive(Debug, Clone)]
pub struct UiBenchResult {
    pub scenario: &'static str,
    pub samples: u32,
    pub avg_layout_ms: f32,
    pub avg_tessellation_ms: f32,
    pub max_total_ms: f32,
}

pub fn run_ui_bench_scenarios(samples: u32) -> Vec<UiBenchResult> {
    let samples = samples.max(1);
    vec![
        run_scenario(samples, "ui-1k-nodes", scenario_many_nodes(1_000)),
        run_scenario(samples, "ui-5k-nodes", scenario_many_nodes(5_000)),
        run_scenario(samples, "ui-splitter-drag", scenario_splitter_drag),
        run_scenario(
            samples,
            "ui-multi-widget-viewport",
            scenario_multi_viewports,
        ),
    ]
}

fn run_scenario(
    samples: u32,
    scenario_name: &'static str,
    mut render_fn: impl FnMut(&egui::Context, u32),
) -> UiBenchResult {
    let ctx = egui::Context::default();
    let mut total_layout_ms = 0.0f32;
    let mut total_tessellation_ms = 0.0f32;
    let mut max_total_ms = 0.0f32;

    for frame in 0..samples {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            time: Some(frame as f64 / 60.0),
            ..Default::default()
        };

        let layout_start = Instant::now();
        let output = ctx.run(raw_input, |ctx| render_fn(ctx, frame));
        let layout_ms = layout_start.elapsed().as_secs_f32() * 1000.0;

        let tess_start = Instant::now();
        let _ = ctx.tessellate(output.shapes, output.pixels_per_point);
        let tessellation_ms = tess_start.elapsed().as_secs_f32() * 1000.0;

        total_layout_ms += layout_ms;
        total_tessellation_ms += tessellation_ms;
        max_total_ms = max_total_ms.max(layout_ms + tessellation_ms);
    }

    UiBenchResult {
        scenario: scenario_name,
        samples,
        avg_layout_ms: total_layout_ms / samples as f32,
        avg_tessellation_ms: total_tessellation_ms / samples as f32,
        max_total_ms,
    }
}

fn scenario_many_nodes(node_count: usize) -> impl FnMut(&egui::Context, u32) {
    move |ctx: &egui::Context, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Nodes: {node_count}"));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in 0..node_count {
                    ui.horizontal(|ui| {
                        ui.label(format!("Item {i}"));
                        let mut enabled = i % 2 == 0;
                        ui.checkbox(&mut enabled, "enabled");
                        let mut value = (i % 100) as f32;
                        ui.add(egui::Slider::new(&mut value, 0.0..=100.0));
                    });
                }
            });
        });
    }
}

fn scenario_splitter_drag(ctx: &egui::Context, frame: u32) {
    let base = 160.0f32;
    let amplitude = 220.0f32;
    let phase = ((frame as f32) * 0.08).sin() * 0.5 + 0.5;
    let width = base + amplitude * phase;

    egui::SidePanel::left("bench_split_left")
        .resizable(true)
        .exact_width(width)
        .show(ctx, |ui| {
            ui.heading("Left");
            for i in 0..128 {
                ui.label(format!("L{i:03}"));
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Center");
        for i in 0..128 {
            ui.label(format!("C{i:03}"));
        }
    });
}

fn scenario_multi_viewports(ctx: &egui::Context, frame: u32) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("4x Widget Realm Viewport");
        let available = ui.available_size();
        let spacing = 8.0f32;
        let cell = egui::vec2(
            ((available.x - spacing) * 0.5).max(32.0),
            ((available.y - spacing) * 0.5).max(32.0),
        );

        for row in 0..2 {
            ui.horizontal(|ui| {
                for col in 0..2 {
                    let index = row * 2 + col;
                    ui.group(|ui| {
                        ui.set_min_size(cell);
                        ui.label(format!("Viewport {}", index + 1));
                        let t = frame as f32 * 0.03 + index as f32;
                        let w = (cell.x - 12.0).max(8.0);
                        let h = (cell.y - 28.0).max(8.0);
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
                        let color = egui::Color32::from_rgb(
                            (110.0 + t.sin() * 40.0) as u8,
                            (140.0 + t.cos() * 30.0) as u8,
                            190,
                        );
                        ui.painter().rect_filled(rect, 4.0, color);
                    });
                }
            });
            if row == 0 {
                ui.add_space(spacing);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::run_ui_bench_scenarios;

    #[test]
    fn ui_bench_scenarios_smoke() {
        let results = run_ui_bench_scenarios(2);
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|result| {
            result.samples == 2
                && !result.scenario.is_empty()
                && result.avg_layout_ms.is_finite()
                && result.avg_layout_ms >= 0.0
                && result.avg_tessellation_ms.is_finite()
                && result.avg_tessellation_ms >= 0.0
                && result.max_total_ms.is_finite()
                && result.max_total_ms >= 0.0
        }));
    }
}
