use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vulfram_types::RealmId;

use crate::{UiDocument, UiDocumentId};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTracedPointerContext {
    pub trace_realm_id: RealmId,
    pub trace_source_realm_id: Option<RealmId>,
    pub uv: Option<glam::Vec2>,
    pub cursor_position: Option<glam::Vec2>,
    pub realm_output_size: Option<glam::UVec2>,
    pub connector_source_size: Option<glam::UVec2>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTracedPointerDispatch {
    pub realm_id: RealmId,
    pub document_id: UiDocumentId,
    pub pos: glam::Vec2,
    pub realm_size: glam::UVec2,
}

pub fn resolve_traced_pointer_dispatch(
    documents: &HashMap<UiDocumentId, UiDocument>,
    context: &UiTracedPointerContext,
) -> Option<UiTracedPointerDispatch> {
    let resolved = resolve_traced_pointer_position(context)?;
    let document_id = hit_test_ui_document(
        documents,
        resolved.realm_id,
        resolved.pos,
        resolved.realm_size,
    )?;
    Some(UiTracedPointerDispatch {
        realm_id: resolved.realm_id,
        document_id,
        pos: resolved.pos,
        realm_size: resolved.realm_size,
    })
}

fn resolve_traced_pointer_position(
    context: &UiTracedPointerContext,
) -> Option<UiTracedPointerPosition> {
    let realm_id = context
        .trace_source_realm_id
        .unwrap_or(context.trace_realm_id);
    let (pos, realm_size) = if let Some(uv) = context.uv {
        let size = if context.trace_source_realm_id.is_some() {
            context.realm_output_size.or(context.connector_source_size)
        } else {
            context.connector_source_size.or(context.realm_output_size)
        }?;
        (uv * size.as_vec2(), size)
    } else if let Some(position) = context.cursor_position {
        let size = context
            .realm_output_size
            .or(context.connector_source_size)
            .unwrap_or(glam::UVec2::new(1, 1));
        (position, size)
    } else {
        return None;
    };

    Some(UiTracedPointerPosition {
        realm_id,
        pos,
        realm_size,
    })
}

fn hit_test_ui_document(
    documents: &HashMap<UiDocumentId, UiDocument>,
    realm_id: RealmId,
    pos: glam::Vec2,
    realm_size: glam::UVec2,
) -> Option<UiDocumentId> {
    let mut best: Option<(i32, UiDocumentId)> = None;
    for document in documents.values() {
        if document.realm_id != realm_id {
            continue;
        }
        let rect = resolve_document_rect(document.rect, realm_size);
        if !document_rect_contains(rect, pos) {
            continue;
        }
        let z_index = document
            .root_children
            .iter()
            .filter_map(|node_id| {
                document
                    .nodes
                    .get(node_id)
                    .and_then(|entry| entry.node.z_index)
            })
            .max()
            .unwrap_or(0);
        let key = (z_index, document.document_id);
        match best {
            Some(current) if key <= current => {}
            _ => best = Some(key),
        }
    }
    best.map(|(_, document_id)| document_id)
}

fn resolve_document_rect(rect: glam::Vec4, realm_size: glam::UVec2) -> glam::Vec4 {
    let max_w = realm_size.x.max(1) as f32;
    let max_h = realm_size.y.max(1) as f32;
    let x = rect.x.max(0.0).min(max_w);
    let y = rect.y.max(0.0).min(max_h);
    let mut w = rect.z;
    let mut h = rect.w;
    if w <= 0.0 {
        w = (max_w - x).max(1.0);
    }
    if h <= 0.0 {
        h = (max_h - y).max(1.0);
    }
    glam::vec4(
        x,
        y,
        w.max(1.0).min((max_w - x).max(1.0)),
        h.max(1.0).min((max_h - y).max(1.0)),
    )
}

fn document_rect_contains(rect: glam::Vec4, pos: glam::Vec2) -> bool {
    let x = rect.x;
    let y = rect.y;
    let w = rect.z.max(1.0);
    let h = rect.w.max(1.0);
    pos.x >= x && pos.y >= y && pos.x <= x + w && pos.y <= y + h
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct UiTracedPointerPosition {
    realm_id: RealmId,
    pos: glam::Vec2,
    realm_size: glam::UVec2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiNode, UiNodeKind, UiNodeProps};

    #[test]
    fn resolve_traced_pointer_dispatch_prefers_source_realm_uv_size() {
        let documents = ui_documents();
        let dispatch = resolve_traced_pointer_dispatch(
            &documents,
            &UiTracedPointerContext {
                trace_realm_id: RealmId(1),
                trace_source_realm_id: Some(RealmId(2)),
                uv: Some(glam::vec2(0.5, 0.25)),
                cursor_position: None,
                realm_output_size: Some(glam::uvec2(200, 100)),
                connector_source_size: Some(glam::uvec2(400, 200)),
            },
        )
        .expect("dispatch should resolve");

        assert_eq!(dispatch.realm_id, RealmId(2));
        assert_eq!(dispatch.document_id, 20);
        assert_eq!(dispatch.pos, glam::vec2(100.0, 25.0));
        assert_eq!(dispatch.realm_size, glam::uvec2(200, 100));
    }

    #[test]
    fn resolve_traced_pointer_dispatch_falls_back_to_cursor_position() {
        let documents = ui_documents();
        let dispatch = resolve_traced_pointer_dispatch(
            &documents,
            &UiTracedPointerContext {
                trace_realm_id: RealmId(1),
                trace_source_realm_id: None,
                uv: None,
                cursor_position: Some(glam::vec2(20.0, 30.0)),
                realm_output_size: Some(glam::uvec2(100, 100)),
                connector_source_size: None,
            },
        )
        .expect("dispatch should resolve");

        assert_eq!(dispatch.realm_id, RealmId(1));
        assert_eq!(dispatch.document_id, 10);
        assert_eq!(dispatch.pos, glam::vec2(20.0, 30.0));
    }

    fn ui_documents() -> HashMap<UiDocumentId, UiDocument> {
        let mut root = UiDocument::new(10, RealmId(1), glam::vec4(0.0, 0.0, 100.0, 100.0));
        root.add_node(None, text_node(1, Some(0)), None)
            .expect("root node");

        let mut source = UiDocument::new(20, RealmId(2), glam::vec4(0.0, 0.0, 200.0, 100.0));
        source
            .add_node(None, text_node(2, Some(1)), None)
            .expect("source node");

        HashMap::from([(10, root), (20, source)])
    }

    fn text_node(node_id: u32, z_index: Option<i32>) -> UiNode {
        UiNode {
            id: node_id,
            kind: UiNodeKind::Text,
            props: UiNodeProps::Text {
                text: "text".into(),
                size: None,
                color: None,
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: None,
            visible: None,
            opacity: None,
            z_index,
        }
    }
}
