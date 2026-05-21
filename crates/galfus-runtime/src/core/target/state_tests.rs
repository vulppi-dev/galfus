use super::DimensionValue;

#[test]
fn dimension_value_px_resolves_directly() {
    let value = DimensionValue::Px(24.0);
    assert_eq!(value.resolve(100.0, 8.0), 24.0);
}

#[test]
fn dimension_value_percent_uses_reference_axis() {
    let value = DimensionValue::Percent(25.0);
    assert_eq!(value.resolve(400.0, 8.0), 100.0);
}

#[test]
fn dimension_value_character_uses_char_width() {
    let value = DimensionValue::Character(10.0);
    assert_eq!(value.resolve(0.0, 7.5), 75.0);
}

#[test]
fn dimension_value_display_uses_four_pixel_grid() {
    let value = DimensionValue::Display(6.0);
    assert_eq!(value.resolve(0.0, 8.0), 24.0);
}

#[test]
fn dimension_value_deserializes_from_host_shape() {
    #[derive(serde::Serialize)]
    struct HostDimension {
        unit: &'static str,
        value: f32,
    }
    let bytes = rmp_serde::to_vec_named(&HostDimension {
        unit: "percent",
        value: 50.0,
    })
    .unwrap();
    let value: DimensionValue = rmp_serde::from_slice(&bytes).unwrap();
    assert_eq!(value, DimensionValue::Percent(50.0));
}
