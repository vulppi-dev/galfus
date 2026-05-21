use super::*;
use crate::core::test_support::test_engine;
use galfus_audio::{AudioSourceParams, AudioStreamState, bind_listener, insert_source};
use glam::Vec3;

#[test]
fn audio_listener_get_returns_binding_when_present() {
    let mut engine = test_engine();
    bind_listener(&mut engine.audio_state, 3, 4);

    let result = engine_cmd_audio_listener_get(&mut engine, &CmdAudioListenerGetArgs {});
    assert!(result.success);
    assert_eq!(result.listener.as_ref().map(|v| v.realm_id), Some(3));
    assert_eq!(result.listener.as_ref().map(|v| v.model_id), Some(4));
}

#[test]
fn audio_source_get_and_list_filter_by_ids() {
    let mut engine = test_engine();
    insert_source(
        &mut engine.audio_state,
        10,
        1,
        2,
        AudioSourceParams {
            position: Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        },
    );
    insert_source(
        &mut engine.audio_state,
        11,
        1,
        3,
        AudioSourceParams::default(),
    );

    let get = engine_cmd_audio_source_get(&mut engine, &CmdAudioSourceGetArgs { source_id: 10 });
    assert!(get.success);
    assert_eq!(get.source.as_ref().map(|v| v.source_id), Some(10));

    let listed = engine_cmd_audio_source_list(
        &mut engine,
        &CmdAudioSourceListArgs {
            source_ids: Some(vec![11]),
        },
    );
    assert!(listed.success);
    assert_eq!(listed.sources.len(), 1);
    assert_eq!(listed.sources[0].source_id, 11);
}

#[test]
fn audio_resource_get_and_list_filter_by_ids() {
    let mut engine = test_engine();
    let mut stream_20 = AudioStreamState::new(8).expect("stream should allocate");
    stream_20
        .apply_chunk(0, &[1, 2, 3, 4])
        .expect("chunk should apply");
    engine.audio_state.streams.insert(20, stream_20);

    let mut stream_21 = AudioStreamState::new(16).expect("stream should allocate");
    stream_21
        .apply_chunk(0, &[1, 2, 3, 4, 5, 6, 7, 8])
        .expect("chunk should apply");
    engine.audio_state.streams.insert(21, stream_21);

    let get =
        engine_cmd_audio_resource_get(&mut engine, &CmdAudioResourceGetArgs { resource_id: 20 });
    assert!(get.success);
    assert_eq!(get.stream.as_ref().map(|v| v.resource_id), Some(20));
    assert_eq!(get.stream.as_ref().map(|v| v.received_bytes), Some(4));

    let listed = engine_cmd_audio_resource_list(
        &mut engine,
        &CmdAudioResourceListArgs {
            resource_ids: Some(vec![21]),
        },
    );
    assert!(listed.success);
    assert_eq!(listed.streams.len(), 1);
    assert_eq!(listed.streams[0].resource_id, 21);
}

#[test]
fn audio_get_returns_not_found_when_source_missing() {
    let mut engine = test_engine();

    let result =
        engine_cmd_audio_source_get(&mut engine, &CmdAudioSourceGetArgs { source_id: 999 });

    assert!(!result.success);
    assert!(result.message.contains("not found"));
}
