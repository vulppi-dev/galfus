use super::*;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HostCmdWindowCloseArgs {
    window_id: u32,
}

#[derive(Serialize)]
struct HostInvalidEnvelope<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    command_type: &'a str,
    content: HostCmdWindowCloseArgs,
}

#[test]
fn send_queue_invalid_type_emits_serialization_error_with_path() {
    let payload = rmp_serde::to_vec_named(&vec![HostInvalidEnvelope {
        id: "invalid-id-type",
        command_type: "cmd-window-close",
        content: HostCmdWindowCloseArgs { window_id: 1 },
    }])
    .expect("host payload serialization must succeed");

    let error = decode_engine_batch_cmds(&payload)
        .expect_err("invalid payload should produce decode error");
    assert!(
        error.contains("at '"),
        "serialization message should include decode path: {error}"
    );
    assert!(
        error.contains("id"),
        "serialization message should mention failing field: {error}"
    );
}
