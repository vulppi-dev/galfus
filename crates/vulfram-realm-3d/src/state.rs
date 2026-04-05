use std::collections::HashMap;

use vulfram_types::RealmId;

#[derive(Debug)]
pub struct RealmEntities<Camera, Model, Light> {
    pub cameras: HashMap<u32, Camera>,
    pub models: HashMap<u32, Model>,
    pub lights: HashMap<u32, Light>,
}

#[derive(Debug, Clone)]
pub struct GeometryRecord<Entry> {
    pub label: Option<String>,
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Realm3dState<Camera, Model, Light, StandardMaterial, PbrMaterial, Geometry, Environment>
{
    pub entities: HashMap<RealmId, RealmEntities<Camera, Model, Light>>,
    pub materials_standard: HashMap<u32, StandardMaterial>,
    pub materials_pbr: HashMap<u32, PbrMaterial>,
    pub geometries: HashMap<u32, Geometry>,
    pub environment_profiles: HashMap<u32, Environment>,
    pub default_environment_id: Option<u32>,
}

impl<Camera, Model, Light> Default for RealmEntities<Camera, Model, Light> {
    fn default() -> Self {
        Self {
            cameras: HashMap::new(),
            models: HashMap::new(),
            lights: HashMap::new(),
        }
    }
}

impl<Camera, Model, Light, StandardMaterial, PbrMaterial, Geometry, Environment> Default
    for Realm3dState<Camera, Model, Light, StandardMaterial, PbrMaterial, Geometry, Environment>
{
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            materials_standard: HashMap::new(),
            materials_pbr: HashMap::new(),
            geometries: HashMap::new(),
            environment_profiles: HashMap::new(),
            default_environment_id: None,
        }
    }
}
