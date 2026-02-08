use std::collections::HashMap;

use crate::core::audio::{AudioListenerBinding, AudioSourceParams, AudioStreamState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RealmId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectorId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PresentId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Generation(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmKind {
    ThreeD,
    TwoD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Onscreen,
    Offscreen,
}

#[derive(Debug, Clone)]
pub struct RealmState {
    pub kind: RealmKind,
    pub output_surface: Option<SurfaceId>,
    pub render_graph: Option<crate::core::render::graph::RenderGraphState>,
    pub flags: u32,
    pub importance: u8,
    pub cache_policy: u8,
    pub last_render_frame: u64,
}

#[derive(Debug, Clone)]
pub struct SurfaceState {
    pub kind: SurfaceKind,
    pub size: glam::UVec2,
    pub format_policy: Option<wgpu::TextureFormat>,
    pub alpha_policy: Option<wgpu::CompositeAlphaMode>,
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ConnectorState {
    pub target_realm: RealmId,
    pub source_surface: SurfaceId,
    pub rect: glam::Vec4,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<glam::Vec4>,
    pub input_flags: u32,
}

#[derive(Debug, Clone)]
pub struct PresentState {
    pub window_id: u32,
    pub surface: SurfaceId,
}

#[derive(Debug, Clone)]
pub struct TableEntry<T> {
    pub generation: Generation,
    pub value: T,
}

impl<T> TableEntry<T> {
    pub fn new(generation: Generation, value: T) -> Self {
        Self { generation, value }
    }
}

#[derive(Debug, Default)]
pub struct RealmTable {
    pub next_id: u32,
    pub entries: HashMap<RealmId, TableEntry<RealmState>>,
}

impl RealmTable {
    pub fn alloc(&mut self, state: RealmState) -> RealmId {
        let id = RealmId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.insert(id, TableEntry::new(Generation(0), state));
        id
    }

    pub fn get(&self, id: RealmId) -> Option<&TableEntry<RealmState>> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: RealmId) -> Option<&mut TableEntry<RealmState>> {
        self.entries.get_mut(&id)
    }

    pub fn remove(&mut self, id: RealmId) -> Option<TableEntry<RealmState>> {
        self.entries.remove(&id)
    }
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
        self.entries
            .insert(id, TableEntry::new(Generation(0), state));
        id
    }

    pub fn get(&self, id: SurfaceId) -> Option<&TableEntry<SurfaceState>> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: SurfaceId) -> Option<&mut TableEntry<SurfaceState>> {
        self.entries.get_mut(&id)
    }

    pub fn remove(&mut self, id: SurfaceId) -> Option<TableEntry<SurfaceState>> {
        self.entries.remove(&id)
    }
}

#[derive(Debug, Default)]
pub struct ConnectorTable {
    pub next_id: u32,
    pub entries: HashMap<ConnectorId, TableEntry<ConnectorState>>,
}

impl ConnectorTable {
    pub fn alloc(&mut self, state: ConnectorState) -> ConnectorId {
        let id = ConnectorId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries
            .insert(id, TableEntry::new(Generation(0), state));
        id
    }

    pub fn get(&self, id: ConnectorId) -> Option<&TableEntry<ConnectorState>> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: ConnectorId) -> Option<&mut TableEntry<ConnectorState>> {
        self.entries.get_mut(&id)
    }

    pub fn remove(&mut self, id: ConnectorId) -> Option<TableEntry<ConnectorState>> {
        self.entries.remove(&id)
    }
}

#[derive(Debug, Default)]
pub struct PresentTable {
    pub next_id: u32,
    pub entries: HashMap<PresentId, TableEntry<PresentState>>,
}

impl PresentTable {
    pub fn alloc(&mut self, state: PresentState) -> PresentId {
        let id = PresentId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries
            .insert(id, TableEntry::new(Generation(0), state));
        id
    }

    pub fn get(&self, id: PresentId) -> Option<&TableEntry<PresentState>> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: PresentId) -> Option<&mut TableEntry<PresentState>> {
        self.entries.get_mut(&id)
    }

    pub fn remove(&mut self, id: PresentId) -> Option<TableEntry<PresentState>> {
        self.entries.remove(&id)
    }

    pub fn remove_by_window(&mut self, window_id: u32) {
        self.entries
            .retain(|_, entry| entry.value.window_id != window_id);
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
pub struct UniversalState {
    pub realms: RealmTable,
    pub surfaces: SurfaceTable,
    pub connectors: ConnectorTable,
    pub presents: PresentTable,
    pub audio: AudioState,
    pub input_routing: InputRoutingState,
    pub surface_cache: SurfaceCache,
    pub frame_report: super::FrameReport,
}

#[derive(Debug, Default)]
pub struct SurfaceCache {
    pub last_good: HashMap<SurfaceId, SurfaceId>,
    pub fallback: HashMap<SurfaceId, SurfaceId>,
}

#[derive(Debug, Default)]
pub struct InputRoutingState {
    pub captures: HashMap<(u32, u64), ConnectorId>,
}
