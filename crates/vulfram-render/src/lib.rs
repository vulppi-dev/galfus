mod cache;
mod graph;
mod graph_fallbacks;
mod profiler;
mod realm_planner;
mod realm_policy;
mod render_bootstrap;
mod target;
mod ui_actions;
mod validation;

pub use cache::{ComputePipelineKey, PipelineKey, RenderCache, ShaderId};
pub use graph::{
    DEFAULT_2D_RENDER_GRAPH_ID, DEFAULT_3D_RENDER_GRAPH_ID, LogicalId, RenderGraphDesc,
    RenderGraphEdge, RenderGraphEdgeReason, RenderGraphLifetime, RenderGraphNode, RenderGraphPlan,
    RenderGraphRecord, RenderGraphResource, RenderGraphResourceKind, RenderGraphState,
    RenderGraphValue, ensure_default_render_graphs, fallback_render_graph_id,
    is_reserved_render_graph_id, render_graph_desc_hash, resolve_render_graph_id, validate_graph,
};
pub use graph_fallbacks::{fallback_graph, ui_fallback_graph};
pub use profiler::{GpuProfiler, GpuTimingReport};
pub use realm_planner::{
    ComposeBlendMode, ComposeConnectorCandidate, ComposeOverlayPlan, ComposeOverlayPlanEntry,
    EnvironmentLayerBinding, ExternalTextureRefreshPlan, ExternalTextureSource,
    RealmEnvironmentBindingPlan, ResolvedSurfaceTarget, SurfaceTargetRequest,
    TargetSizeUpdatePlanEntry, TargetSizeUpdateRequest, build_soft_cut_diagnostic,
    build_target_surface_map, collect_connectors_by_realm, collect_cut_connectors,
    collect_window_camera_target_sizes, map_realms_to_windows, plan_compose_overlays,
    plan_external_texture_refresh, plan_realm_environment_bindings, plan_surface_targets,
    plan_target_size_updates, resolve_connector_surface, resolve_realm_surface,
    should_render_realm, update_present_size_cache, update_surface_cache,
};
pub use realm_policy::{
    clear_alpha_for_realm_kind, graph_is_compatible_with_realm_kind,
    supports_render_pass_for_realm_kind,
};
pub use render_bootstrap::{
    RenderAdapterBootstrapInfo, RenderDeviceFeaturePlan, RenderSurfaceConfigPlan, analyze_adapter,
    build_default_instance_descriptor, build_device_descriptor, create_default_instance,
    plan_device_features, plan_surface_config, resolve_rgba16f_msaa_supported_mask,
};
pub use target::{RenderTarget, ensure_render_target};
pub use ui_actions::{UiPlatformAction, collect_platform_actions};
