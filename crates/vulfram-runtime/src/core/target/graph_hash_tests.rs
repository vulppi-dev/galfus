use super::*;
use crate::core::realm::{RealmKind, RealmState};
use crate::core::target::{TargetKind, TargetLayerLayout};

fn target_state(kind: TargetKind, window_id: Option<u32>) -> TargetState {
    TargetState {
        kind,
        window_id,
        size: Some(glam::UVec2::new(64, 64)),
        format_policy: Some(wgpu::TextureFormat::Rgba16Float),
        alpha_policy: Some(wgpu::CompositeAlphaMode::Auto),
        msaa_samples: Some(1),
    }
}

fn layer_state(realm_id: u32, target_id: TargetId, z_index: i32) -> TargetLayerState {
    TargetLayerState {
        realm_id,
        target_id,
        layout: TargetLayerLayout {
            z_index,
            ..TargetLayerLayout::default()
        },
        camera_id: None,
        environment_id: None,
    }
}

#[test]
fn global_hash_is_order_independent() {
    let mut realms_a = RealmTable::default();
    let _r0 = realms_a.alloc(RealmState {
        kind: RealmKind::TwoD,
        output_surface: None,
        render_graph_id: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });

    let mut targets_a = HashMap::new();
    targets_a.insert(TargetId(2), target_state(TargetKind::Texture, None));
    targets_a.insert(TargetId(1), target_state(TargetKind::Window, Some(1)));

    let mut layers_a = HashMap::new();
    layers_a.insert((0, TargetId(2)), layer_state(0, TargetId(2), 1));
    layers_a.insert((0, TargetId(1)), layer_state(0, TargetId(1), 0));

    let hash_a = hash_targets_layers_and_realms(&targets_a, &layers_a, &realms_a);

    let mut realms_b = RealmTable::default();
    let _r0b = realms_b.alloc(RealmState {
        kind: RealmKind::TwoD,
        output_surface: None,
        render_graph_id: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });

    let mut targets_b = HashMap::new();
    targets_b.insert(TargetId(1), target_state(TargetKind::Window, Some(1)));
    targets_b.insert(TargetId(2), target_state(TargetKind::Texture, None));

    let mut layers_b = HashMap::new();
    layers_b.insert((0, TargetId(1)), layer_state(0, TargetId(1), 0));
    layers_b.insert((0, TargetId(2)), layer_state(0, TargetId(2), 1));

    let hash_b = hash_targets_layers_and_realms(&targets_b, &layers_b, &realms_b);
    assert_eq!(hash_a, hash_b);
}

#[test]
fn entry_hashes_change_when_target_changes() {
    let realms = RealmTable::default();
    let mut targets = HashMap::new();
    targets.insert(TargetId(1), target_state(TargetKind::Texture, None));
    let layers = HashMap::new();

    let (target_hashes_a, _, _) = hash_entries(&targets, &layers, &realms);

    if let Some(target) = targets.get_mut(&TargetId(1)) {
        target.msaa_samples = Some(4);
    }

    let (target_hashes_b, _, _) = hash_entries(&targets, &layers, &realms);
    assert_ne!(
        target_hashes_a.get(&TargetId(1)),
        target_hashes_b.get(&TargetId(1))
    );
}
