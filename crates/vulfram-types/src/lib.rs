use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RealmId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SurfaceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectorId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PresentId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RealmKind {
    ThreeD,
    TwoD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceKind {
    Onscreen,
    Offscreen,
}

pub type UiThemeId = u32;
pub type UiFontId = u32;
pub type UiImageId = u32;
pub type UiDocumentId = u32;
pub type UiNodeId = u32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_id_round_trips_through_messagepack() {
        let encoded = rmp_serde::to_vec_named(&RealmId(42)).expect("realm id should encode");
        let decoded: RealmId = rmp_serde::from_slice(&encoded).expect("realm id should decode");
        assert_eq!(decoded, RealmId(42));
    }

    #[test]
    fn realm_kind_uses_kebab_case_strings() {
        let encoded = serde_json::to_string(&RealmKind::ThreeD).expect("enum should encode");
        assert_eq!(encoded, "\"three-d\"");
    }

    #[test]
    fn surface_kind_distinguishes_onscreen_and_offscreen() {
        assert_ne!(SurfaceKind::Onscreen, SurfaceKind::Offscreen);
    }
}
