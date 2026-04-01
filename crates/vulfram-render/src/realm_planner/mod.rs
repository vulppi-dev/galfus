mod common;
mod compose;
mod targets;
#[cfg(test)]
mod tests;

pub use common::{
    EnvironmentLayerBinding, RealmEnvironmentBindingPlan, build_soft_cut_diagnostic,
    build_target_surface_map, collect_connectors_by_realm, collect_cut_connectors,
    collect_window_camera_target_sizes, map_realms_to_windows, plan_realm_environment_bindings,
    resolve_realm_surface, should_render_realm, update_present_size_cache, update_surface_cache,
};
pub use compose::{
    ComposeBlendMode, ComposeConnectorCandidate, ComposeOverlayPlan, ComposeOverlayPlanEntry,
    plan_compose_overlays, resolve_connector_surface,
};
pub use targets::{
    ExternalTextureRefreshPlan, ExternalTextureSource, ResolvedSurfaceTarget, SurfaceTargetRequest,
    TargetSizeUpdatePlanEntry, TargetSizeUpdateRequest, plan_external_texture_refresh,
    plan_surface_targets, plan_target_size_updates,
};
