use super::types::{
    CmdAudioResourceDisposeArgs, CmdAudioResourceUpsertArgs, CmdResultAudioResourceDispose,
    CmdResultAudioResourceUpsert, audio_disabled_message,
};
use crate::core::buffers::state::UploadType;
use crate::core::state::EngineState;
use galfus_audio::upsert_stream_chunk;

pub fn engine_cmd_audio_resource_upsert(
    engine: &mut EngineState,
    args: &CmdAudioResourceUpsertArgs,
) -> CmdResultAudioResourceUpsert {
    if !engine.audio_available {
        return CmdResultAudioResourceUpsert {
            success: false,
            message: audio_disabled_message(),
            pending: false,
            received_bytes: 0,
            total_bytes: 0,
            complete: false,
        };
    }
    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultAudioResourceUpsert {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
                pending: false,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            };
        }
    };

    if buffer.upload_type != UploadType::BinaryAsset {
        return CmdResultAudioResourceUpsert {
            success: false,
            message: format!(
                "Invalid buffer type. Expected BinaryAsset, got {:?}",
                buffer.upload_type
            ),
            pending: false,
            received_bytes: 0,
            total_bytes: 0,
            complete: false,
        };
    }

    let offset = args.offset_bytes.unwrap_or(0);
    let has_stream = engine.audio_state.streams.contains_key(&args.resource_id);
    let is_stream_upsert = args.total_bytes.is_some() || has_stream;

    if is_stream_upsert {
        upsert_stream_resource(engine, args, offset, buffer.data)
    } else {
        match engine
            .audio
            .buffer_create_from_bytes(args.resource_id, buffer.data)
        {
            Ok(()) => CmdResultAudioResourceUpsert {
                success: true,
                message: "Audio buffer queued".into(),
                pending: true,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            },
            Err(message) => CmdResultAudioResourceUpsert {
                success: false,
                message,
                pending: false,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            },
        }
    }
}

fn upsert_stream_resource(
    engine: &mut EngineState,
    args: &CmdAudioResourceUpsertArgs,
    offset: u64,
    data: Vec<u8>,
) -> CmdResultAudioResourceUpsert {
    let stream_update = match upsert_stream_chunk(
        &mut engine.audio_state,
        args.resource_id,
        args.total_bytes,
        offset,
        &data,
    ) {
        Ok(result) => result,
        Err(message) => {
            let (received_bytes, total_bytes, complete) = engine
                .audio_state
                .streams
                .get(&args.resource_id)
                .map(|stream| (stream.received_bytes, stream.total_bytes, stream.complete()))
                .unwrap_or((0, 0, false));
            return CmdResultAudioResourceUpsert {
                success: false,
                message,
                pending: false,
                received_bytes,
                total_bytes,
                complete,
            };
        }
    };
    let complete = stream_update.complete;
    engine
        .runtime
        .push_event(crate::core::cmd::EngineEvent::System(
            crate::core::system::events::SystemEvent::AudioStreamProgress {
                resource_id: args.resource_id,
                received_bytes: stream_update.received_bytes,
                total_bytes: stream_update.total_bytes,
                complete,
            },
        ));
    if complete {
        let Some(completed_data) = stream_update.completed_data else {
            return CmdResultAudioResourceUpsert {
                success: false,
                message: format!(
                    "Audio stream {} completed without payload",
                    args.resource_id
                ),
                pending: false,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            };
        };
        match engine
            .audio
            .buffer_create_from_bytes(args.resource_id, completed_data)
        {
            Ok(()) => CmdResultAudioResourceUpsert {
                success: true,
                message: "Audio stream queued".into(),
                pending: true,
                received_bytes: stream_update.received_bytes,
                total_bytes: stream_update.total_bytes,
                complete: true,
            },
            Err(message) => CmdResultAudioResourceUpsert {
                success: false,
                message,
                pending: false,
                received_bytes: stream_update.received_bytes,
                total_bytes: stream_update.total_bytes,
                complete: true,
            },
        }
    } else {
        CmdResultAudioResourceUpsert {
            success: true,
            message: "Audio stream chunk queued".into(),
            pending: true,
            received_bytes: stream_update.received_bytes,
            total_bytes: stream_update.total_bytes,
            complete: false,
        }
    }
}

pub fn engine_cmd_audio_resource_dispose(
    engine: &mut EngineState,
    args: &CmdAudioResourceDisposeArgs,
) -> CmdResultAudioResourceDispose {
    if !engine.audio_available {
        return CmdResultAudioResourceDispose {
            success: false,
            message: audio_disabled_message(),
        };
    }
    engine.audio_state.streams.remove(&args.resource_id);
    match engine.audio.buffer_dispose(args.resource_id) {
        Ok(()) => CmdResultAudioResourceDispose {
            success: true,
            message: "Resource disposed".into(),
        },
        Err(message) => CmdResultAudioResourceDispose {
            success: false,
            message,
        },
    }
}
