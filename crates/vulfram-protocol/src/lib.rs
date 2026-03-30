use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::{Display, Formatter};

pub use vulfram_types as types;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCodecError {
    message: String,
}

impl ProtocolCodecError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ProtocolCodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ProtocolCodecError {}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSimple {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CmdNotificationSendArgs {
    pub id: Option<String>,
    pub title: String,
    pub body: String,
    pub level: NotificationLevel,
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultNotificationSend {
    pub success: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdSystemBuildVersionGetArgs {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultSystemBuildVersionGet {
    pub success: bool,
    pub message: String,
    pub build_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope<T> {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResponseEnvelope<T> {
    pub id: u64,
    #[serde(flatten)]
    pub response: T,
}

pub fn decode_named<T>(data: &[u8]) -> Result<T, ProtocolCodecError>
where
    T: DeserializeOwned,
{
    let mut deserializer = rmp_serde::Deserializer::new(data);
    serde_path_to_error::deserialize::<_, T>(&mut deserializer).map_err(|error| {
        let path = error.path().to_string();
        let inner = error.into_inner();
        if path.is_empty() {
            ProtocolCodecError::new(format!("invalid MessagePack payload: {inner}"))
        } else {
            ProtocolCodecError::new(format!("invalid MessagePack payload at '{path}': {inner}"))
        }
    })
}

pub fn encode_named<T>(value: &T) -> Result<Vec<u8>, ProtocolCodecError>
where
    T: Serialize,
{
    rmp_serde::to_vec_named(value)
        .map_err(|error| ProtocolCodecError::new(format!("failed to encode payload: {error}")))
}

#[cfg(test)]
mod tests {
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
        let decoded: CommandEnvelope<TestCmd> =
            decode_named(&encoded).expect("payload should decode");

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
}
