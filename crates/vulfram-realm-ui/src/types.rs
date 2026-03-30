use serde::{Deserialize, Serialize};
pub use vulfram_types::{UiDocumentId, UiFontId, UiImageId, UiNodeId, UiThemeId};

use crate::UiNodeProps;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UiThemeValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiNodeKind {
    Container,
    Window,
    Panel,
    SplitPane,
    Area,
    Frame,
    ScrollArea,
    Grid,
    Popup,
    Tooltip,
    Modal,
    Resize,
    Scene,
    Canvas,
    Text,
    RichText,
    Link,
    Hyperlink,
    Button,
    Checkbox,
    Radio,
    SelectableLabel,
    Toggle,
    Slider,
    DragValue,
    ProgressBar,
    ComboBox,
    MenuButton,
    CollapsingHeader,
    ImageButton,
    Spinner,
    TextEdit,
    Input,
    Image,
    WidgetRealmViewport,
    Separator,
    Spacer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiLayoutDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
    Grid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiAlign {
    Start,
    Center,
    End,
    Stretch,
}

impl Default for UiAlign {
    fn default() -> Self {
        UiAlign::Start
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UiLength {
    Auto,
    Fill,
    Px(f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiPanelKind {
    SideLeft,
    SideRight,
    Top,
    Bottom,
    Central,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiSplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiStroke {
    pub width: f32,
    pub color: UiColor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiWindowAnchor {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiSize {
    pub width: UiLength,
    pub height: UiLength,
}

impl Default for UiSize {
    fn default() -> Self {
        Self {
            width: UiLength::Auto,
            height: UiLength::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiLayout {
    pub direction: UiLayoutDirection,
    #[serde(default)]
    pub align: UiAlign,
    #[serde(default)]
    pub justify: UiAlign,
    #[serde(default)]
    pub gap: f32,
    #[serde(default)]
    pub columns: Option<u32>,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub wrap_limit: Option<f32>,
}

impl Default for UiLayout {
    fn default() -> Self {
        Self {
            direction: UiLayoutDirection::Column,
            align: UiAlign::Start,
            justify: UiAlign::Start,
            gap: 0.0,
            columns: None,
            wrap: false,
            wrap_limit: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiTextAlign {
    LeftTop,
    LeftCenter,
    LeftBottom,
    CenterTop,
    CenterCenter,
    CenterBottom,
    RightTop,
    RightCenter,
    RightBottom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiNode {
    pub id: UiNodeId,
    pub kind: UiNodeKind,
    pub props: UiNodeProps,
    #[serde(default)]
    pub tooltip: Option<String>,
    #[serde(default)]
    pub context_menu: Option<Vec<String>>,
    #[serde(default)]
    pub anim: Option<UiAnim>,
    #[serde(default)]
    pub display: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiAnim {
    #[serde(default)]
    pub opacity: Option<UiAnimSpec>,
    #[serde(default)]
    pub translate_y: Option<UiAnimSpec>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiAnimSpec {
    pub from: f32,
    pub to: f32,
    pub duration_ms: u32,
    #[serde(default)]
    pub easing: UiAnimEasing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiAnimEasing {
    Linear,
    EaseInOut,
}

impl Default for UiAnimEasing {
    fn default() -> Self {
        UiAnimEasing::Linear
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiOp {
    Add {
        parent: Option<UiNodeId>,
        node: UiNode,
        index: Option<u32>,
    },
    Remove {
        node_id: UiNodeId,
    },
    Clear {
        parent: Option<UiNodeId>,
    },
    Set {
        node_id: UiNodeId,
        props: UiNodeProps,
    },
    Move {
        node_id: UiNodeId,
        new_parent: Option<UiNodeId>,
        index: Option<u32>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_layout_defaults_to_column_start() {
        let layout = UiLayout::default();
        assert_eq!(layout.direction, UiLayoutDirection::Column);
        assert_eq!(layout.align, UiAlign::Start);
        assert_eq!(layout.justify, UiAlign::Start);
    }

    #[test]
    fn ui_eventful_node_round_trips_through_json() {
        let node = UiNode {
            id: 10,
            kind: UiNodeKind::Text,
            props: UiNodeProps::Text {
                text: "hello".into(),
                size: Some(14.0),
                color: None,
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: Some(true),
            visible: Some(true),
            opacity: Some(1.0),
            z_index: Some(2),
        };

        let json = serde_json::to_string(&node).expect("node should encode");
        let decoded: UiNode = serde_json::from_str(&json).expect("node should decode");
        assert_eq!(decoded, node);
    }
}
