use super::{PostProcessConfig, SkyboxConfig};

#[test]
fn default_postprocess_is_neutral() {
    let config = PostProcessConfig::default();
    assert!(config.filter_enabled);
    assert_eq!(config.filter_exposure, 1.0);
    assert_eq!(config.filter_gamma, 1.0);
    assert_eq!(config.filter_saturation, 1.0);
    assert_eq!(config.filter_contrast, 1.0);
    assert_eq!(config.filter_tonemap_mode, 0);
    assert_eq!(config.filter_vignette, 0.0);
    assert_eq!(config.filter_grain, 0.0);
    assert_eq!(config.filter_chromatic_aberration, 0.0);
    assert_eq!(config.filter_blur, 0.0);
    assert_eq!(config.filter_sharpen, 0.0);
}

#[test]
fn default_skybox_supports_multi_sun_controls() {
    let config = SkyboxConfig::default();
    assert_eq!(config.horizon_ground_threshold, 0.45);
    assert_eq!(config.horizon_sky_threshold, 0.55);
    assert!(config.directional_lights.is_empty());
}
