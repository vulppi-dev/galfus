use std::collections::HashMap;

use crate::core::input::InteractionRuntimeState;
use crate::core::render::SceneRuntimeState;
use crate::core::target::TargetRoutingState;
pub use vulfram_audio::AudioState;
pub use vulfram_realm_core::{
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
pub struct RealmCompositionState {
    pub realms: RealmTable,
    pub surfaces: SurfaceTable,
    pub connectors: ConnectorTable,
    pub presents: PresentTable,
    pub surface_cache: SurfaceCache,
    pub frame_report: super::FrameReport,
}

#[derive(Debug, Default)]
pub struct UniversalState {
    pub composition: RealmCompositionState,
    pub targets: TargetRoutingState,
    pub interaction: InteractionRuntimeState,
    pub scene: SceneRuntimeState,
}

pub use vulfram_input::{
    InputCapture, InputRoutingCache, InputRoutingConnectorHit, InputRoutingState,
};
