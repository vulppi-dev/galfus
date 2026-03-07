use serde::{Deserialize, Serialize};

use super::{
    UiColor, UiImageId, UiLayout, UiPadding, UiPanelKind, UiSize, UiSplitDirection, UiStroke,
    UiTextAlign, UiWindowAnchor,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiPaintStroke {
    pub width: f32,
    pub color: UiColor,
    #[serde(default)]
    pub join: Option<String>,
    #[serde(default)]
    pub cap: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiPaintOp {
    LineSegment {
        from: glam::Vec2,
        to: glam::Vec2,
        stroke: UiPaintStroke,
    },
    Polyline {
        points: Vec<glam::Vec2>,
        stroke: UiPaintStroke,
    },
    Rect {
        min: glam::Vec2,
        max: glam::Vec2,
        rounding: Option<f32>,
        stroke: UiPaintStroke,
    },
    RectFilled {
        min: glam::Vec2,
        max: glam::Vec2,
        rounding: Option<f32>,
        fill: UiColor,
    },
    Circle {
        center: glam::Vec2,
        radius: f32,
        stroke: UiPaintStroke,
    },
    CircleFilled {
        center: glam::Vec2,
        radius: f32,
        fill: UiColor,
    },
    ConvexPolygon {
        points: Vec<glam::Vec2>,
        fill: UiColor,
        #[serde(default)]
        stroke: Option<UiPaintStroke>,
    },
    QuadraticBezier {
        from: glam::Vec2,
        ctrl: glam::Vec2,
        to: glam::Vec2,
        steps: Option<u32>,
        stroke: UiPaintStroke,
    },
    CubicBezier {
        from: glam::Vec2,
        ctrl1: glam::Vec2,
        ctrl2: glam::Vec2,
        to: glam::Vec2,
        steps: Option<u32>,
        stroke: UiPaintStroke,
    },
    Text {
        position: glam::Vec2,
        text: String,
        #[serde(default)]
        size: Option<f32>,
        color: UiColor,
        #[serde(default)]
        align: Option<UiTextAlign>,
    },
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
    Canvas {
        ops: Vec<UiPaintOp>,
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        clip: Option<bool>,
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
