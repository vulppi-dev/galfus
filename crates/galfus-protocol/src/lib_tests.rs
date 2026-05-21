use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
enum TestCmd {
    Ping { value: u32 },
}

#[test]
fn command_envelope_round_trips() {
    let payload = CommandEnvelope {
        id: 7,
        cmd: TestCmd::Ping { value: 9 },
    };

    let encoded = encode_named(&payload).expect("payload should encode");
    let decoded: CommandEnvelope<TestCmd> = decode_named(&encoded).expect("payload should decode");

    assert_eq!(decoded, payload);
}

#[test]
fn decode_named_reports_failing_path() {
    #[derive(Serialize)]
    struct InvalidEnvelope<'a> {
        id: &'a str,
        #[serde(rename = "type")]
        command_type: &'a str,
        content: serde_json::Value,
    }

    let invalid = vec![InvalidEnvelope {
        id: "oops",
        command_type: "ping",
        content: serde_json::json!({ "value": 9 }),
    }];

    let encoded = rmp_serde::to_vec_named(&invalid).expect("invalid payload should encode");
    let error = decode_named::<Vec<CommandEnvelope<TestCmd>>>(&encoded)
        .expect_err("payload should fail to decode");

    assert!(
        error.to_string().contains("id"),
        "decode error should mention failing field: {error}"
    );
}

#[test]
fn notification_args_round_trip_through_named_codec() {
    let payload = CmdNotificationSendArgs {
        id: Some("notif-1".into()),
        title: "Hello".into(),
        body: "World".into(),
        level: NotificationLevel::Success,
        timeout: Some(3000),
    };

    let encoded = encode_named(&payload).expect("notification args should encode");
    let decoded: CmdNotificationSendArgs =
        decode_named(&encoded).expect("notification args should decode");

    assert_eq!(decoded, payload);
}

#[test]
fn window_create_args_apply_default_size() {
    let decoded: CmdWindowCreateArgs =
        serde_json::from_str(r#"{ "windowId": 1 }"#).expect("window create args should decode");
    assert_eq!(decoded.size, glam::UVec2::new(800, 600));
    assert_eq!(decoded.initial_state, EngineWindowState::Windowed);
}
