use super::{
    AudioListenerBinding, AudioSourceParams, AudioState, AudioStreamState, bind_listener,
    dispose_listener_for_realm, dispose_source, insert_source, snapshot_audio_state, update_source,
    upsert_stream_chunk,
};
use glam::{Quat, Vec3};

#[test]
fn audio_stream_merges_overlapping_chunks_without_double_counting() {
    let mut stream = AudioStreamState::new(8).expect("stream");
    assert_eq!(
        stream.apply_chunk(0, &[1, 2, 3, 4]).expect("first chunk"),
        4
    );
    assert_eq!(stream.apply_chunk(2, &[9, 9, 9, 9]).expect("overlap"), 2);
    assert_eq!(stream.received_bytes, 6);
    assert_eq!(stream.ranges, vec![(0, 6)]);
}

#[test]
fn audio_stream_reports_complete_only_after_full_range() {
    let mut stream = AudioStreamState::new(4).expect("stream");
    assert!(!stream.complete());
    let _ = stream.apply_chunk(0, &[1, 2, 3, 4]).expect("full chunk");
    assert!(stream.complete());
}

#[test]
fn snapshot_audio_state_sorts_sources_and_streams() {
    let mut state = AudioState {
        listener_binding: Some(AudioListenerBinding {
            realm_id: 9,
            model_id: 7,
        }),
        ..Default::default()
    };
    state.source_params.insert(
        20,
        AudioSourceParams {
            position: Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        },
    );
    state.source_bindings.insert(
        20,
        AudioListenerBinding {
            realm_id: 5,
            model_id: 4,
        },
    );
    state.source_params.insert(10, AudioSourceParams::default());
    let mut stream = AudioStreamState::new(2).expect("stream");
    let _ = stream.apply_chunk(0, &[1, 2]).expect("chunk");
    state.streams.insert(3, stream);

    let snapshot = snapshot_audio_state(&state, true, true, true);

    assert_eq!(snapshot.listener.expect("listener").realm_id, 9);
    assert_eq!(snapshot.sources.len(), 2);
    assert_eq!(snapshot.sources[0].source_id, 10);
    assert_eq!(snapshot.sources[1].source_id, 20);
    assert_eq!(snapshot.sources[1].realm_id, Some(5));
    assert_eq!(snapshot.streams[0].resource_id, 3);
    assert!(snapshot.streams[0].complete);
}

#[test]
fn update_source_mutates_params_and_binding() {
    let mut state = AudioState::default();
    insert_source(&mut state, 1, 10, 20, AudioSourceParams::default());

    let params = update_source(
        &mut state,
        1,
        Some(30),
        Some(40),
        Some(Vec3::new(1.0, 2.0, 3.0)),
        None,
        Some(Quat::IDENTITY),
        Some(0.5),
        Some(1.5),
        None,
    )
    .expect("updated");

    assert_eq!(params.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(params.gain, 0.5);
    assert_eq!(params.pitch, 1.5);
    assert_eq!(state.source_bindings.get(&1).expect("binding").realm_id, 30);
    dispose_source(&mut state, 1);
    assert!(!state.source_params.contains_key(&1));
}

#[test]
fn listener_binding_helpers_replace_and_dispose() {
    let mut state = AudioState::default();
    bind_listener(&mut state, 7, 9);
    assert_eq!(state.listener_binding.expect("listener").realm_id, 7);
    assert!(dispose_listener_for_realm(&mut state, 7));
    assert!(state.listener_binding.is_none());
}

#[test]
fn upsert_stream_chunk_returns_completed_payload_once() {
    let mut state = AudioState::default();
    let first = upsert_stream_chunk(&mut state, 5, Some(4), 0, &[1, 2]).expect("first chunk");
    assert!(!first.complete);
    assert!(first.completed_data.is_none());

    let second = upsert_stream_chunk(&mut state, 5, None, 2, &[3, 4]).expect("second chunk");
    assert!(second.complete);
    assert_eq!(
        second.completed_data.expect("completed data"),
        vec![1, 2, 3, 4]
    );
    assert!(!state.streams.contains_key(&5));
}
