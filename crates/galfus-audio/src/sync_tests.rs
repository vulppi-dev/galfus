use glam::{Quat, Vec3};

use crate::{AudioSourceParams, AudioState, bind_listener, insert_source};

use super::{AudioModelTransform, plan_bound_source_updates, resolve_listener_binding_state};

#[test]
fn resolve_listener_binding_state_reads_transform_from_bound_model() {
    let mut state = AudioState::default();
    bind_listener(&mut state, 4, 9);
    let listener = resolve_listener_binding_state(
        &state,
        4,
        &[AudioModelTransform {
            model_id: 9,
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
        }],
    )
    .expect("listener");
    assert_eq!(listener.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(listener.forward, Vec3::NEG_Z);
    assert_eq!(listener.up, Vec3::Y);
}

#[test]
fn plan_bound_source_updates_tracks_bound_models_and_listener_model() {
    let mut state = AudioState::default();
    bind_listener(&mut state, 7, 10);
    insert_source(&mut state, 2, 7, 10, AudioSourceParams::default());
    insert_source(&mut state, 1, 7, 11, AudioSourceParams::default());

    let updates = plan_bound_source_updates(
        &state,
        7,
        &[
            AudioModelTransform {
                model_id: 10,
                translation: Vec3::new(5.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
            },
            AudioModelTransform {
                model_id: 11,
                translation: Vec3::new(9.0, 1.0, 0.0),
                rotation: Quat::IDENTITY,
            },
        ],
    );

    assert_eq!(updates.len(), 2);
    assert_eq!(updates[0].source_id, 1);
    assert_eq!(updates[0].params.position, Vec3::new(9.0, 1.0, 0.0));
    assert_eq!(updates[1].source_id, 2);
    assert_eq!(updates[1].params.position, Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(updates[1].params.spatial.min_distance, 0.0);
    assert_eq!(updates[1].params.spatial.max_distance, 0.01);
    assert_eq!(updates[1].params.spatial.rolloff, 0.0);
}
