use std::collections::HashMap;

pub use vulfram_types::{ConnectorId, PresentId, RealmId, RealmKind, SurfaceId};

#[derive(Debug, Clone)]
pub struct RealmState {
    pub kind: RealmKind,
    pub output_surface: Option<SurfaceId>,
    pub render_graph_id: Option<u32>,
    pub importance: u8,
    pub cache_policy: u8,
    pub last_render_frame: u64,
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
    pub value: T,
}

impl<T> TableEntry<T> {
    pub fn new(value: T) -> Self {
        Self { value }
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
        self.entries.insert(id, TableEntry::new(state));
        id
    }

    pub fn get(&self, id: RealmId) -> Option<&TableEntry<RealmState>> {
        self.entries.get(&id)
    }

    pub fn remove(&mut self, id: RealmId) -> Option<TableEntry<RealmState>> {
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
        self.entries.insert(id, TableEntry::new(state));
        id
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
        self.entries.insert(id, TableEntry::new(state));
        id
    }

    pub fn remove(&mut self, id: PresentId) -> Option<TableEntry<PresentState>> {
        self.entries.remove(&id)
    }

    pub fn remove_by_window(&mut self, window_id: u32) {
        self.entries
            .retain(|_, entry| entry.value.window_id != window_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_table_allocates_monotonic_ids() {
        let mut table = RealmTable::default();
        let first = table.alloc(RealmState {
            kind: RealmKind::ThreeD,
            output_surface: None,
            render_graph_id: None,
            importance: 1,
            cache_policy: 0,
            last_render_frame: 0,
        });
        let second = table.alloc(RealmState {
            kind: RealmKind::TwoD,
            output_surface: Some(SurfaceId(3)),
            render_graph_id: Some(9),
            importance: 2,
            cache_policy: 1,
            last_render_frame: 7,
        });

        assert_eq!(first, RealmId(0));
        assert_eq!(second, RealmId(1));
    }

    #[test]
    fn present_table_remove_by_window_prunes_matching_entries() {
        let mut table = PresentTable::default();
        let keep = table.alloc(PresentState {
            window_id: 1,
            surface: SurfaceId(10),
        });
        let _drop = table.alloc(PresentState {
            window_id: 2,
            surface: SurfaceId(20),
        });

        table.remove_by_window(2);

        assert!(table.entries.contains_key(&keep));
        assert_eq!(table.entries.len(), 1);
    }
}
