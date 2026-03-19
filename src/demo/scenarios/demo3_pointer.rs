use glam::{Mat4, Vec2};

use crate::core::cmd::EngineCmd;
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{UiNode, UiNodeKind, UiNodeProps, UiOp};
use crate::core::window::{CmdWindowCursorArgs, CursorGrabMode, CursorIcon};

use super::DemoIds;

pub(super) fn demo3_mode_from_key(key_code: u32) -> Option<Demo3PointerMode> {
    match key_code {
        6 => Some(Demo3PointerMode::Normal),
        7 => Some(Demo3PointerMode::Locked),
        8 => Some(Demo3PointerMode::Confined),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Demo3PointerMode {
    Normal,
    Locked,
    Confined,
}

impl Demo3PointerMode {
    fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Locked => "locked",
            Self::Confined => "confined",
        }
    }

    fn to_cursor_mode(self) -> CursorGrabMode {
        match self {
            Self::Normal => CursorGrabMode::None,
            Self::Locked => CursorGrabMode::Locked,
            Self::Confined => CursorGrabMode::Confined,
        }
    }
}

pub(super) struct Demo3State {
    mode: Demo3PointerMode,
    pointer_capture_active: bool,
    pointer_position: Vec2,
    pointer_delta: Vec2,
    last_pointer_position: Option<Vec2>,
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
    ui_document_id: u32,
    ui_container_id: u32,
    ui_text_id: u32,
    ui_version: u64,
    last_ui_update_ms: u64,
    ui_dirty: bool,
}

impl Demo3State {
    pub(super) fn new(ids: DemoIds) -> Self {
        Self {
            mode: Demo3PointerMode::Normal,
            pointer_capture_active: false,
            pointer_position: Vec2::ZERO,
            pointer_delta: Vec2::ZERO,
            last_pointer_position: None,
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.004,
            ui_document_id: ids.ui_doc_extra,
            ui_container_id: ids.ui_node_extra,
            ui_text_id: ids.ui_node_extra + 1,
            ui_version: 1,
            last_ui_update_ms: 0,
            ui_dirty: true,
        }
    }

    pub(super) fn setup_commands(&self, ui_realm_id: u32) -> Vec<EngineCmd> {
        vec![
            EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
                document_id: self.ui_document_id,
                realm_id: ui_realm_id,
                rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
                theme_id: None,
            }),
            EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                document_id: self.ui_document_id,
                version: self.ui_version,
                ops: vec![
                    UiOp::Add {
                        parent: None,
                        node: UiNode {
                            id: self.ui_container_id,
                            kind: UiNodeKind::Area,
                            props: UiNodeProps::Area {
                                label: Some("demo3-pointer-debug".into()),
                                x: Some(12.0),
                                y: Some(42.0),
                                draggable: Some(false),
                                size: None,
                            },
                            tooltip: None,
                            context_menu: None,
                            anim: None,
                            display: None,
                            visible: None,
                            opacity: None,
                            z_index: Some(1100),
                        },
                        index: None,
                    },
                    UiOp::Add {
                        parent: Some(self.ui_container_id),
                        node: UiNode {
                            id: self.ui_text_id,
                            kind: UiNodeKind::Text,
                            props: UiNodeProps::Text {
                                text: self.ui_text(),
                                size: Some(16.0),
                                color: None,
                            },
                            tooltip: None,
                            context_menu: None,
                            anim: None,
                            display: None,
                            visible: None,
                            opacity: None,
                            z_index: Some(1101),
                        },
                        index: None,
                    },
                ],
            }),
        ]
    }

    pub(super) fn cursor_command(window_id: u32, mode: Demo3PointerMode) -> EngineCmd {
        EngineCmd::CmdWindowCursor(CmdWindowCursorArgs {
            window_id,
            visible: Some(mode != Demo3PointerMode::Locked),
            mode: Some(mode.to_cursor_mode()),
            icon: Some(CursorIcon::Crosshair),
        })
    }

    pub(super) fn set_mode(&mut self, mode: Demo3PointerMode) -> bool {
        if self.mode == mode {
            return false;
        }
        self.mode = mode;
        self.pointer_capture_active = mode == Demo3PointerMode::Confined;
        self.pointer_delta = Vec2::ZERO;
        self.last_pointer_position = None;
        self.ui_dirty = true;
        true
    }

    pub(super) fn update_capture_active(&mut self, active: bool) {
        if self.pointer_capture_active == active {
            return;
        }
        self.pointer_capture_active = active;
        self.pointer_delta = Vec2::ZERO;
        self.last_pointer_position = None;
        self.ui_dirty = true;
    }

    fn rotation_enabled(&self) -> bool {
        match self.mode {
            Demo3PointerMode::Normal => false,
            Demo3PointerMode::Confined => true,
            Demo3PointerMode::Locked => self.pointer_capture_active,
        }
    }

    pub(super) fn on_pointer_move(&mut self, position: Vec2) {
        let delta = self
            .last_pointer_position
            .map(|prev| position - prev)
            .unwrap_or(Vec2::ZERO);
        self.pointer_position = position;
        self.pointer_delta = delta;
        self.last_pointer_position = Some(position);
        self.ui_dirty = true;

        if !self.rotation_enabled() {
            return;
        }

        self.yaw += delta.x * self.sensitivity;
        self.pitch = (self.pitch - delta.y * self.sensitivity).clamp(-1.45, 1.45);
    }

    fn ui_text(&self) -> String {
        format!(
            "Mode: {}{}\nPointer: ({:.2}, {:.2})\nDelta: ({:.2}, {:.2})\nControles: [1] normal [2] locked [3] confined",
            self.mode.label(),
            if self.mode == Demo3PointerMode::Locked && !self.pointer_capture_active {
                " (aguardando capture)"
            } else {
                ""
            },
            self.pointer_position.x,
            self.pointer_position.y,
            self.pointer_delta.x,
            self.pointer_delta.y
        )
    }

    pub(super) fn frame_commands(&mut self, total_ms: u64) -> Vec<EngineCmd> {
        if !self.ui_dirty && total_ms.saturating_sub(self.last_ui_update_ms) < 100 {
            return Vec::new();
        }
        self.ui_dirty = false;
        self.last_ui_update_ms = total_ms;
        self.ui_version = self.ui_version.saturating_add(1);
        vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            document_id: self.ui_document_id,
            version: self.ui_version,
            ops: vec![UiOp::Set {
                node_id: self.ui_text_id,
                props: UiNodeProps::Text {
                    text: self.ui_text(),
                    size: Some(16.0),
                    color: None,
                },
            }],
        })]
    }

    pub(super) fn model_transform(&self) -> Mat4 {
        Mat4::from_rotation_y(self.yaw) * Mat4::from_rotation_x(self.pitch)
    }
}
