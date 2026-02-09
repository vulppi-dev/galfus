use glam::Vec4;
use serde::{Deserialize, Serialize};

use crate::core::realm::{ConnectorId, ConnectorState, RealmId, SurfaceId};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdConnectorCreateArgs {
    pub target_realm_id: u32,
    pub source_surface_id: u32,
    pub rect: Vec4,
    #[serde(default)]
    pub z_index: i32,
    #[serde(default)]
    pub blend_mode: u32,
    #[serde(default)]
    pub clip: Option<Vec4>,
    #[serde(default)]
    pub input_flags: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultConnectorCreate {
    pub success: bool,
    pub message: String,
    pub connector_id: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdConnectorDisposeArgs {
    pub connector_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultConnectorDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_connector_create(
    engine: &mut EngineState,
    args: &CmdConnectorCreateArgs,
) -> CmdResultConnectorCreate {
    let target_realm = RealmId(args.target_realm_id);
    if engine.universal_state.realms.get(target_realm).is_none() {
        return CmdResultConnectorCreate {
            success: false,
            message: format!("Realm {} not found", args.target_realm_id),
            connector_id: None,
        };
    }

    let source_surface = SurfaceId(args.source_surface_id);
    if engine.universal_state.surfaces.get(source_surface).is_none() {
        return CmdResultConnectorCreate {
            success: false,
            message: format!("Surface {} not found", args.source_surface_id),
            connector_id: None,
        };
    }

    let connector_id = engine.universal_state.connectors.alloc(ConnectorState {
        target_realm,
        source_surface,
        rect: args.rect,
        z_index: args.z_index,
        blend_mode: args.blend_mode,
        clip: args.clip,
        input_flags: args.input_flags,
    });

    CmdResultConnectorCreate {
        success: true,
        message: "Connector created".into(),
        connector_id: Some(connector_id.0),
    }
}

pub fn engine_cmd_connector_dispose(
    engine: &mut EngineState,
    args: &CmdConnectorDisposeArgs,
) -> CmdResultConnectorDispose {
    let connector_id = ConnectorId(args.connector_id);
    if engine.universal_state.connectors.remove(connector_id).is_none() {
        return CmdResultConnectorDispose {
            success: false,
            message: format!("Connector {} not found", args.connector_id),
        };
    }

    engine
        .universal_state
        .input_routing
        .captures
        .retain(|_, capture| capture.connector_id != connector_id);

    CmdResultConnectorDispose {
        success: true,
        message: "Connector disposed".into(),
    }
}
