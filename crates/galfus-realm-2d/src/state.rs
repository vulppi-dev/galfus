use std::collections::HashMap;

use galfus_types::RealmId;

#[derive(Debug)]
pub struct RealmEntities<Camera, Sprite, Shape> {
    pub cameras: HashMap<u32, Camera>,
    pub sprites: HashMap<u32, Sprite>,
    pub shapes: HashMap<u32, Shape>,
}

#[derive(Debug)]
pub struct Realm2dState<Camera, Sprite, Shape, Material> {
    pub entities: HashMap<RealmId, RealmEntities<Camera, Sprite, Shape>>,
    pub materials: HashMap<u32, Material>,
}

impl<Camera, Sprite, Shape> Default for RealmEntities<Camera, Sprite, Shape> {
    fn default() -> Self {
        Self {
            cameras: HashMap::new(),
            sprites: HashMap::new(),
            shapes: HashMap::new(),
        }
    }
}

impl<Camera, Sprite, Shape, Material> Default for Realm2dState<Camera, Sprite, Shape, Material> {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            materials: HashMap::new(),
        }
    }
}
