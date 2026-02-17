use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub type UiThemeId = u32;
#[allow(dead_code)]
pub type UiFontId = u32;
#[allow(dead_code)]
pub type UiImageId = u32;
#[allow(dead_code)]
pub type UiDocumentId = u32;
#[allow(dead_code)]
pub type UiNodeId = u32;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiNodeProps {
    Container {
        #[serde(default)]
        layout: UiLayout,
        #[serde(default)]
        padding: Option<UiPadding>,
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        scroll_x: bool,
        #[serde(default)]
        scroll_y: bool,
    },
    Window {
        title: String,
        #[serde(default)]
        open: Option<bool>,
        #[serde(default)]
        movable: Option<bool>,
        #[serde(default)]
        resizable: Option<bool>,
        #[serde(default)]
        collapsible: Option<bool>,
        #[serde(default)]
        anchored: Option<UiWindowAnchor>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Panel {
        kind: UiPanelKind,
        #[serde(default)]
        resizable: Option<bool>,
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        min_size: Option<f32>,
        #[serde(default)]
        max_size: Option<f32>,
    },
    SplitPane {
        direction: UiSplitDirection,
        #[serde(default)]
        ratio: Option<f32>,
        #[serde(default)]
        resizable: Option<bool>,
        #[serde(default)]
        min_a: Option<f32>,
        #[serde(default)]
        max_a: Option<f32>,
        #[serde(default)]
        min_b: Option<f32>,
        #[serde(default)]
        max_b: Option<f32>,
    },
    Area {
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        x: Option<f32>,
        #[serde(default)]
        y: Option<f32>,
        #[serde(default)]
        draggable: Option<bool>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Frame {
        #[serde(default)]
        padding: Option<UiPadding>,
        #[serde(default)]
        fill: Option<UiColor>,
        #[serde(default)]
        stroke: Option<UiStroke>,
        #[serde(default)]
        rounding: Option<f32>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    ScrollArea {
        #[serde(default)]
        scroll_x: bool,
        #[serde(default)]
        scroll_y: bool,
        #[serde(default)]
        auto_shrink: Option<bool>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Grid {
        #[serde(default)]
        columns: Option<u32>,
        #[serde(default)]
        striped: Option<bool>,
        #[serde(default)]
        min_col_width: Option<f32>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Popup {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        open: Option<bool>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Tooltip {
        text: String,
    },
    Modal {
        title: String,
        #[serde(default)]
        open: Option<bool>,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Resize {
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        min_size: Option<UiSize>,
        #[serde(default)]
        max_size: Option<UiSize>,
    },
    Scene {
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        zoom_min: Option<f32>,
        #[serde(default)]
        zoom_max: Option<f32>,
        #[serde(default)]
        pan_enabled: Option<bool>,
    },
    Text {
        text: String,
        #[serde(default)]
        size: Option<f32>,
        #[serde(default)]
        color: Option<UiColor>,
    },
    RichText {
        text: String,
        #[serde(default)]
        size: Option<f32>,
        #[serde(default)]
        color: Option<UiColor>,
        #[serde(default)]
        strong: Option<bool>,
        #[serde(default)]
        italics: Option<bool>,
        #[serde(default)]
        underline: Option<bool>,
        #[serde(default)]
        strikethrough: Option<bool>,
        #[serde(default)]
        monospace: Option<bool>,
    },
    Link {
        label: String,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Hyperlink {
        label: String,
        url: String,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Button {
        label: String,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Checkbox {
        label: String,
        checked: bool,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Radio {
        label: String,
        selected: bool,
        #[serde(default)]
        enabled: Option<bool>,
    },
    SelectableLabel {
        label: String,
        selected: bool,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Toggle {
        label: String,
        value: bool,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Slider {
        value: f64,
        min: f64,
        max: f64,
        #[serde(default)]
        step: Option<f64>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    DragValue {
        value: f64,
        #[serde(default)]
        speed: Option<f64>,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        suffix: Option<String>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    ProgressBar {
        value: f64,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        animate: Option<bool>,
        #[serde(default)]
        show_percentage: Option<bool>,
    },
    ComboBox {
        label: String,
        selected: String,
        options: Vec<String>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    MenuButton {
        label: String,
        #[serde(default)]
        enabled: Option<bool>,
    },
    CollapsingHeader {
        label: String,
        #[serde(default)]
        open: Option<bool>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    ImageButton {
        source: UiImageSource,
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Spinner {
        #[serde(default)]
        size: Option<f32>,
    },
    TextEdit {
        value: String,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        multiline: Option<bool>,
        #[serde(default)]
        password: Option<bool>,
        #[serde(default)]
        char_limit: Option<usize>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Input {
        value: String,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Image {
        source: UiImageSource,
        #[serde(default)]
        size: Option<UiSize>,
    },
    WidgetRealmViewport {
        target_id: u64,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Separator,
    Spacer {
        width: Option<f32>,
        height: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiImageSource {
    UiImage(UiImageId),
    Target(u64),
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
