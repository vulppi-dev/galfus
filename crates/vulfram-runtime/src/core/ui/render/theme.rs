pub(super) fn apply_theme(ctx: &egui::Context, theme: &crate::core::ui::state::UiThemeState) {
    let mut style = (*ctx.style()).clone();
    if theme_bool(theme, "darkMode").unwrap_or(false) {
        style.visuals = egui::Visuals::dark();
    } else if theme.data.contains_key("darkMode") {
        style.visuals = egui::Visuals::light();
    }

    if let Some(size) = theme_float(theme, "fontSize") {
        for text_style in style.text_styles.values_mut() {
            text_style.size = size;
        }
    }
    if let Some(size) = theme_float(theme, "fontHeading") {
        set_text_style_size(&mut style, egui::TextStyle::Heading, size);
    }
    if let Some(size) = theme_float(theme, "fontBody") {
        set_text_style_size(&mut style, egui::TextStyle::Body, size);
    }
    if let Some(size) = theme_float(theme, "fontMonospace") {
        set_text_style_size(&mut style, egui::TextStyle::Monospace, size);
    }
    if let Some(size) = theme_float(theme, "fontButton") {
        set_text_style_size(&mut style, egui::TextStyle::Button, size);
    }
    if let Some(size) = theme_float(theme, "fontSmall") {
        set_text_style_size(&mut style, egui::TextStyle::Small, size);
    }

    if let Some(color) = theme_color(theme, "textColor") {
        style.visuals.override_text_color = Some(color);
    }
    if let Some(color) = theme_color(theme, "panelFill") {
        style.visuals.panel_fill = color;
    }
    if let Some(color) = theme_color(theme, "windowFill") {
        style.visuals.window_fill = color;
    }
    if let Some(color) = theme_color(theme, "accentColor") {
        style.visuals.selection.bg_fill = color;
    }
    if let Some(color) = theme_color(theme, "selectionStrokeColor") {
        style.visuals.selection.stroke.color = color;
    }
    if let Some(color) = theme_color(theme, "hyperlinkColor") {
        style.visuals.hyperlink_color = color;
    }

    if let Some(value) = theme_float(theme, "spacingItemX") {
        style.spacing.item_spacing.x = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingItemY") {
        style.spacing.item_spacing.y = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingButtonX") {
        style.spacing.button_padding.x = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingButtonY") {
        style.spacing.button_padding.y = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingWindowX") {
        style.spacing.window_margin.left = value.max(0.0);
        style.spacing.window_margin.right = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingWindowY") {
        style.spacing.window_margin.top = value.max(0.0);
        style.spacing.window_margin.bottom = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingIndent") {
        style.spacing.indent = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingInteractX") {
        style.spacing.interact_size.x = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingInteractY") {
        style.spacing.interact_size.y = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingSliderWidth") {
        style.spacing.slider_width = value.max(0.0);
    }

    if let Some(value) = theme_float(theme, "roundingWindow") {
        style.visuals.window_rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingMenu") {
        style.visuals.menu_rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetInactive") {
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetHovered") {
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetActive") {
        style.visuals.widgets.active.rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetOpen") {
        style.visuals.widgets.open.rounding = egui::Rounding::same(value.max(0.0));
    }

    if let Some(value) = theme_float(theme, "strokeWindowWidth") {
        style.visuals.window_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWindowColor") {
        style.visuals.window_stroke.color = color;
    }
    if let Some(value) = theme_float(theme, "strokeWidgetInactiveWidth") {
        style.visuals.widgets.inactive.bg_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWidgetInactiveColor") {
        style.visuals.widgets.inactive.bg_stroke.color = color;
    }
    if let Some(value) = theme_float(theme, "strokeWidgetHoveredWidth") {
        style.visuals.widgets.hovered.bg_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWidgetHoveredColor") {
        style.visuals.widgets.hovered.bg_stroke.color = color;
    }
    if let Some(value) = theme_float(theme, "strokeWidgetActiveWidth") {
        style.visuals.widgets.active.bg_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWidgetActiveColor") {
        style.visuals.widgets.active.bg_stroke.color = color;
    }

    apply_text_style_family_override(&mut style, theme, "fontFamilyProportional", false);
    apply_text_style_family_override(&mut style, theme, "fontFamilyMonospace", true);

    ctx.set_style(style);

    if !theme.font_data.is_empty() || !theme.font_families.is_empty() {
        let mut definitions = egui::FontDefinitions::default();
        for (name, bytes) in &theme.font_data {
            definitions
                .font_data
                .insert(name.clone(), egui::FontData::from_owned(bytes.clone()));
        }
        for (family_key, family_fonts) in &theme.font_families {
            let family = if family_key.eq_ignore_ascii_case("proportional") {
                egui::FontFamily::Proportional
            } else if family_key.eq_ignore_ascii_case("monospace") {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Name(family_key.clone().into())
            };
            definitions.families.insert(family, family_fonts.clone());
        }
        ctx.set_fonts(definitions);
    }
}

fn parse_color_string(value: &str) -> Option<egui::Color32> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        let parsed = match hex.len() {
            6 => u32::from_str_radix(hex, 16).ok().map(|v| (v, 255u8)),
            8 => u32::from_str_radix(hex, 16)
                .ok()
                .map(|v| (v >> 8, (v & 0xFF) as u8)),
            _ => None,
        };
        if let Some((rgb, a)) = parsed {
            let r = ((rgb >> 16) & 0xFF) as u8;
            let g = ((rgb >> 8) & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;
            return Some(egui::Color32::from_rgba_premultiplied(r, g, b, a));
        }
    }

    let parts: Vec<_> = trimmed.split(',').map(|p| p.trim()).collect();
    if parts.len() >= 3 {
        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;
        let a = parts
            .get(3)
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(255);
        return Some(egui::Color32::from_rgba_premultiplied(r, g, b, a));
    }

    None
}

fn theme_float(theme: &crate::core::ui::state::UiThemeState, key: &str) -> Option<f32> {
    match theme.data.get(key) {
        Some(crate::core::ui::types::UiThemeValue::Float(value)) => Some(*value as f32),
        Some(crate::core::ui::types::UiThemeValue::Int(value)) => Some(*value as f32),
        _ => None,
    }
}

fn theme_bool(theme: &crate::core::ui::state::UiThemeState, key: &str) -> Option<bool> {
    match theme.data.get(key) {
        Some(crate::core::ui::types::UiThemeValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn theme_color(theme: &crate::core::ui::state::UiThemeState, key: &str) -> Option<egui::Color32> {
    match theme.data.get(key) {
        Some(crate::core::ui::types::UiThemeValue::String(value)) => parse_color_string(value),
        _ => None,
    }
}

fn set_text_style_size(style: &mut egui::Style, text_style: egui::TextStyle, size: f32) {
    if let Some(font_id) = style.text_styles.get_mut(&text_style) {
        font_id.size = size.max(1.0);
    }
}

fn apply_text_style_family_override(
    style: &mut egui::Style,
    theme: &crate::core::ui::state::UiThemeState,
    key: &str,
    monospace_only: bool,
) {
    let Some(crate::core::ui::types::UiThemeValue::String(family_name)) = theme.data.get(key)
    else {
        return;
    };
    let family = if family_name.eq_ignore_ascii_case("proportional") {
        egui::FontFamily::Proportional
    } else if family_name.eq_ignore_ascii_case("monospace") {
        egui::FontFamily::Monospace
    } else {
        egui::FontFamily::Name(family_name.clone().into())
    };
    for (text_style, font_id) in style.text_styles.iter_mut() {
        let is_mono_style = matches!(text_style, egui::TextStyle::Monospace);
        if (monospace_only && is_mono_style) || (!monospace_only && !is_mono_style) {
            font_id.family = family.clone();
        }
    }
}
