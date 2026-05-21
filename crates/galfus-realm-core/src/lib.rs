mod realm_graph;
mod render_passes;
mod report;
mod state;
mod tables;
mod target_graph;

pub use galfus_types::{ConnectorId, PresentId, RealmId, RealmKind, SurfaceId};
pub use realm_graph::{RealmGraphEdge, RealmGraphPlan, RealmGraphPlanner};
pub use render_passes::{
    RENDER_PASS_2D_BATCH, RENDER_PASS_2D_COMPOSE, RENDER_PASS_2D_DRAW, RENDER_PASS_2D_PREPARE,
    RENDER_PASS_BLOOM, RENDER_PASS_COMPOSE, RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL,
    RENDER_PASS_OUTLINE, RENDER_PASS_POST, RENDER_PASS_SHADOW, RENDER_PASS_SKYBOX,
    RENDER_PASS_SSAO, RENDER_PASS_SSAO_BLUR, RENDER_PASS_UI,
};
pub use report::{
    FrameCutEdge, FrameReport, SurfaceCacheEntry, TargetAutoLinkFailure, TargetCutEdge,
    TargetInvocationReport, TargetLayerReportKey,
};
pub use state::{
    AutoLink, ConnectorState, DimensionValue, PresentState, RealmState, RenderInvocation,
    SurfaceCache, TargetEdge, TargetGraphDiff, TargetGraphPlan, TargetGraphPlanner, TargetId,
    TargetKind, TargetLayerLayout, TargetLayerState,
};
pub use tables::{ConnectorTable, PresentTable, RealmTable, TableEntry};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
