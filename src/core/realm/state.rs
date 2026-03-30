use std::collections::HashMap;

use crate::core::audio::{AudioListenerBinding, AudioSourceParams, AudioStreamState};
use crate::core::resources::{
    CameraNode, EnvironmentConfig, ForwardAtlasEntry, GeometryPrimitiveType, LightRecord,
    MaterialPbrRecord, MaterialStandardRecord, ModelRecord, TargetTextureBinding, TextureRecord,
};
use crate::core::target::{TargetGraphCache, TargetLayerTable, TargetTable};
use crate::core::ui::UiState;
pub use vulfram_scene_core::{
    AutoLink, ConnectorState, ConnectorTable, PresentState, PresentTable, RealmState, RealmTable,
    SurfaceCache, TableEntry,
};
pub use vulfram_types::{ConnectorId, PresentId, RealmId, RealmKind, SurfaceId, SurfaceKind};

#[derive(Debug, Clone)]
pub struct SurfaceState {
    pub kind: SurfaceKind,
    pub size: glam::UVec2,
    pub format_policy: Option<wgpu::TextureFormat>,
    pub alpha_policy: Option<wgpu::CompositeAlphaMode>,
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Default)]
pub struct SurfaceTable {
    pub next_id: u32,
    pub entries: HashMap<SurfaceId, TableEntry<SurfaceState>>,
}

impl SurfaceTable {
    pub fn alloc(&mut self, state: SurfaceState) -> SurfaceId {
        let id = SurfaceId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.insert(id, TableEntry::new(state));
        id
    }

    pub fn get(&self, id: SurfaceId) -> Option<&TableEntry<SurfaceState>> {
        self.entries.get(&id)
    }

    pub fn remove(&mut self, id: SurfaceId) -> Option<TableEntry<SurfaceState>> {
        self.entries.remove(&id)
    }
}

#[derive(Debug, Default)]
pub struct AudioState {
    pub listener_binding: Option<AudioListenerBinding>,
    pub source_bindings: HashMap<u32, AudioListenerBinding>,
    pub source_params: HashMap<u32, AudioSourceParams>,
    pub streams: HashMap<u32, AudioStreamState>,
}

#[derive(Debug, Default)]
pub struct RealmEntities {
    pub cameras: HashMap<u32, CameraNode>,
    pub models: HashMap<u32, ModelRecord>,
    pub lights: HashMap<u32, LightRecord>,
}

#[derive(Debug, Clone)]
pub struct UniversalGeometryRecord {
    pub label: Option<String>,
    pub entries: Vec<(GeometryPrimitiveType, Vec<u8>)>,
}

#[derive(Debug, Default)]
pub struct UniversalResources {
    pub materials_standard: HashMap<u32, MaterialStandardRecord>,
    pub materials_pbr: HashMap<u32, MaterialPbrRecord>,
    pub textures: HashMap<u32, TextureRecord>,
    pub forward_atlas_entries: HashMap<u32, ForwardAtlasEntry>,
    pub target_texture_binds: HashMap<u32, TargetTextureBinding>,
    pub geometries: HashMap<u32, UniversalGeometryRecord>,
}

#[derive(Debug, Default)]
pub struct UniversalState {
    pub realms: RealmTable,
    pub surfaces: SurfaceTable,
    pub connectors: ConnectorTable,
    pub presents: PresentTable,
    pub ui: UiState,
    pub targets: TargetTable,
    pub target_layers: TargetLayerTable,
    pub target_graph_cache: TargetGraphCache,
    pub auto_links: std::collections::HashMap<(u32, crate::core::target::TargetId), AutoLink>,
    pub host_realm_index: HashMap<u32, RealmId>,
    pub target_ui_realm_index: HashMap<crate::core::target::TargetId, RealmId>,
    pub target_autolink_failures: Vec<super::TargetAutoLinkFailure>,
    pub environment_profiles: HashMap<u32, EnvironmentConfig>,
    pub default_environment_id: Option<u32>,
    pub input_routing: InputRoutingState,
    pub target_listeners: crate::core::input::listeners::InputTargetListenerStore,
    pub surface_cache: SurfaceCache,
    pub frame_report: super::FrameReport,
    pub realm_entities: HashMap<RealmId, RealmEntities>,
    pub render_graphs: HashMap<u32, crate::core::render::graph::RenderGraphRecord>,
    pub render_graph_plan_cache: HashMap<u64, crate::core::render::graph::RenderGraphState>,
    pub universal_resources: UniversalResources,
}

pub use vulfram_input::{
    InputCapture, InputRoutingCache, InputRoutingConnectorHit, InputRoutingState,
};
