use std::collections::HashMap;

use crate::realm_planner::AutoGraphLinkPlan;
use vulfram_realm_core::{RealmId, RealmKind, SurfaceId, TargetKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoGraphExistingLink {
    pub surface_id: SurfaceId,
    pub has_connector: bool,
    pub has_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoGraphSurfaceSyncOp {
    Allocate,
    Update,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoGraphLinkSyncOp {
    Create,
    Rebuild,
    UpdateConnectorLayout,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoGraphLayerSyncPlan {
    pub surface_op: AutoGraphSurfaceSyncOp,
    pub link_op: AutoGraphLinkSyncOp,
    pub desired_link: AutoGraphLinkPlan,
}

pub fn plan_auto_graph_layer_sync(
    target_kind: TargetKind,
    target_window_id: Option<u32>,
    source_realm_id: RealmId,
    source_realm_kind: RealmKind,
    host_realm_index: &HashMap<u32, RealmId>,
    current_surface_id: Option<SurfaceId>,
    current_surface_matches: bool,
    is_primary: bool,
    existing_link: Option<AutoGraphExistingLink>,
) -> AutoGraphLayerSyncPlan {
    let desired_link = crate::realm_planner::plan_auto_graph_link(
        target_kind,
        target_window_id,
        source_realm_id,
        source_realm_kind,
        host_realm_index,
    );
    let surface_op = match current_surface_id {
        None => AutoGraphSurfaceSyncOp::Allocate,
        Some(_) if is_primary && !current_surface_matches => AutoGraphSurfaceSyncOp::Update,
        Some(_) => AutoGraphSurfaceSyncOp::Keep,
    };

    let link_op = match existing_link {
        None => AutoGraphLinkSyncOp::Create,
        Some(existing_link) => {
            let wants_connector = matches!(desired_link, AutoGraphLinkPlan::Connector { .. });
            let wants_present = matches!(desired_link, AutoGraphLinkPlan::Present { .. });
            let surface_changed =
                current_surface_id.is_none_or(|surface_id| surface_id != existing_link.surface_id);
            let shape_changed = wants_connector != existing_link.has_connector
                || wants_present != existing_link.has_present;

            if surface_changed || shape_changed {
                AutoGraphLinkSyncOp::Rebuild
            } else if wants_connector {
                AutoGraphLinkSyncOp::UpdateConnectorLayout
            } else {
                AutoGraphLinkSyncOp::Keep
            }
        }
    };

    AutoGraphLayerSyncPlan {
        surface_op,
        link_op,
        desired_link,
    }
}
