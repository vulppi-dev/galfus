use crate::core::cmd::EngineCmd;
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{UiNode, UiNodeKind, UiNodeProps, UiOp};

pub struct FpsHud {
    pub document_id: u32,
    node_id: u32,
    version: u64,
    last_update_ms: u64,
    last_fps: f32,
}

impl FpsHud {
    pub fn new(demo_number: u32) -> Self {
        let base = 90_000 + demo_number * 16;
        Self {
            document_id: base,
            node_id: base + 1,
            version: 1,
            last_update_ms: 0,
            last_fps: 0.0,
        }
    }

    pub fn setup_commands(&self, realm_id: u32) -> Vec<EngineCmd> {
        vec![
            EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
                document_id: self.document_id,
                realm_id,
                rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
                theme_id: None,
            }),
            EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                document_id: self.document_id,
                version: self.version,
                ops: vec![UiOp::Add {
                    parent: None,
                    node: UiNode {
                        id: self.node_id,
                        kind: UiNodeKind::Text,
                        props: UiNodeProps::Text {
                            text: "FPS: --".into(),
                            size: Some(18.0),
                            color: None,
                        },
                        tooltip: None,
                        context_menu: None,
                        anim: None,
                        display: None,
                        visible: None,
                        opacity: None,
                        z_index: Some(999),
                    },
                    index: None,
                }],
            }),
        ]
    }

    pub fn frame_commands(&mut self, total_ms: u64, delta_ms: u32) -> Vec<EngineCmd> {
        if delta_ms > 0 {
            self.last_fps = 1000.0 / delta_ms as f32;
        }

        if total_ms.saturating_sub(self.last_update_ms) < 200 {
            return Vec::new();
        }

        self.last_update_ms = total_ms;
        self.version = self.version.saturating_add(1);
        vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            document_id: self.document_id,
            version: self.version,
            ops: vec![UiOp::Set {
                node_id: self.node_id,
                props: UiNodeProps::Text {
                    text: format!("FPS: {:.1}", self.last_fps),
                    size: Some(18.0),
                    color: None,
                },
            }],
        })]
    }
}
