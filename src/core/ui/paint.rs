use crate::core::ui::types::{UiColor, UiPaintOp, UiPaintStroke, UiTextAlign};

pub fn paint_ops(ui: &egui::Ui, rect: egui::Rect, ops: &[UiPaintOp], clip_enabled: bool) {
    let painter = if clip_enabled {
        ui.painter_at(rect)
    } else {
        ui.painter().clone()
    };
    for op in ops {
        match op {
            UiPaintOp::LineSegment { from, to, stroke } => {
                painter.line_segment([to_pos(rect, *from), to_pos(rect, *to)], to_stroke(stroke));
            }
            UiPaintOp::Polyline { points, stroke } => {
                if points.len() >= 2 {
                    let points: Vec<egui::Pos2> = points.iter().map(|p| to_pos(rect, *p)).collect();
                    painter.add(egui::Shape::line(points, to_stroke(stroke)));
                }
            }
            UiPaintOp::Rect {
                min,
                max,
                rounding,
                stroke,
            } => {
                painter.rect_stroke(
                    egui::Rect::from_min_max(to_pos(rect, *min), to_pos(rect, *max)),
                    rounding.unwrap_or(0.0).max(0.0),
                    to_stroke(stroke),
                );
            }
            UiPaintOp::RectFilled {
                min,
                max,
                rounding,
                fill,
            } => {
                painter.rect_filled(
                    egui::Rect::from_min_max(to_pos(rect, *min), to_pos(rect, *max)),
                    rounding.unwrap_or(0.0).max(0.0),
                    to_color(*fill),
                );
            }
            UiPaintOp::Circle {
                center,
                radius,
                stroke,
            } => {
                painter.circle_stroke(to_pos(rect, *center), radius.max(0.0), to_stroke(stroke));
            }
            UiPaintOp::CircleFilled {
                center,
                radius,
                fill,
            } => {
                painter.circle_filled(to_pos(rect, *center), radius.max(0.0), to_color(*fill));
            }
            UiPaintOp::ConvexPolygon {
                points,
                fill,
                stroke,
            } => {
                if points.len() >= 3 {
                    let points: Vec<egui::Pos2> = points.iter().map(|p| to_pos(rect, *p)).collect();
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        to_color(*fill),
                        stroke.as_ref().map(to_stroke).unwrap_or(egui::Stroke::NONE),
                    ));
                }
            }
            UiPaintOp::QuadraticBezier {
                from,
                ctrl,
                to,
                steps,
                stroke,
            } => {
                let steps = steps.unwrap_or(24).max(3);
                let mut points = Vec::with_capacity(steps as usize + 1);
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let p = quadratic(*from, *ctrl, *to, t);
                    points.push(to_pos(rect, p));
                }
                painter.add(egui::Shape::line(points, to_stroke(stroke)));
            }
            UiPaintOp::CubicBezier {
                from,
                ctrl1,
                ctrl2,
                to,
                steps,
                stroke,
            } => {
                let steps = steps.unwrap_or(32).max(4);
                let mut points = Vec::with_capacity(steps as usize + 1);
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let p = cubic(*from, *ctrl1, *ctrl2, *to, t);
                    points.push(to_pos(rect, p));
                }
                painter.add(egui::Shape::line(points, to_stroke(stroke)));
            }
            UiPaintOp::Text {
                position,
                text,
                size,
                color,
                align,
            } => {
                let align = map_align(align.unwrap_or(UiTextAlign::LeftTop));
                let font = size
                    .map(|v| egui::FontId::proportional(v.max(1.0)))
                    .unwrap_or_else(|| egui::TextStyle::Body.resolve(ui.style()));
                painter.text(to_pos(rect, *position), align, text, font, to_color(*color));
            }
        }
    }
}

fn to_pos(rect: egui::Rect, local: glam::Vec2) -> egui::Pos2 {
    egui::pos2(rect.min.x + local.x, rect.min.y + local.y)
}

fn to_color(color: UiColor) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

fn to_stroke(stroke: &UiPaintStroke) -> egui::Stroke {
    egui::Stroke::new(stroke.width.max(0.0), to_color(stroke.color))
}

fn quadratic(a: glam::Vec2, b: glam::Vec2, c: glam::Vec2, t: f32) -> glam::Vec2 {
    let mt = 1.0 - t;
    mt * mt * a + 2.0 * mt * t * b + t * t * c
}

fn cubic(a: glam::Vec2, b: glam::Vec2, c: glam::Vec2, d: glam::Vec2, t: f32) -> glam::Vec2 {
    let mt = 1.0 - t;
    mt * mt * mt * a + 3.0 * mt * mt * t * b + 3.0 * mt * t * t * c + t * t * t * d
}

fn map_align(align: UiTextAlign) -> egui::Align2 {
    match align {
        UiTextAlign::LeftTop => egui::Align2::LEFT_TOP,
        UiTextAlign::LeftCenter => egui::Align2::LEFT_CENTER,
        UiTextAlign::LeftBottom => egui::Align2::LEFT_BOTTOM,
        UiTextAlign::CenterTop => egui::Align2::CENTER_TOP,
        UiTextAlign::CenterCenter => egui::Align2::CENTER_CENTER,
        UiTextAlign::CenterBottom => egui::Align2::CENTER_BOTTOM,
        UiTextAlign::RightTop => egui::Align2::RIGHT_TOP,
        UiTextAlign::RightCenter => egui::Align2::RIGHT_CENTER,
        UiTextAlign::RightBottom => egui::Align2::RIGHT_BOTTOM,
    }
}
