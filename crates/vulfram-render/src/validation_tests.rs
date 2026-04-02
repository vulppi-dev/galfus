use std::collections::HashMap;

use crate::{
    LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphLifetime,
    RenderGraphNode, RenderGraphResource, RenderGraphResourceKind, RenderGraphState,
    validate_graph,
};
use vulfram_realm_core::{
    RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL, RENDER_PASS_POST, RENDER_PASS_SHADOW,
    RENDER_PASS_SKYBOX,
};

fn id(name: &str) -> LogicalId {
    LogicalId::Str(name.into())
}

fn resource(name: &str) -> RenderGraphResource {
    RenderGraphResource {
        res_id: id(name),
        kind: RenderGraphResourceKind::Texture,
        lifetime: RenderGraphLifetime::Frame,
        alias_group: None,
    }
}

#[test]
fn fallback_graph_remains_semantically_valid() {
    let fallback = crate::fallback_graph();
    let result = validate_graph(&fallback);
    assert!(result.is_ok());
    assert!(RenderGraphState::from_desc(fallback).is_ok());
}

#[test]
fn rejects_input_without_any_producer() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![RenderGraphNode {
            node_id: id("n0"),
            pass_id: RENDER_PASS_FORWARD.into(),
            inputs: vec![id("missing")],
            outputs: vec![],
            params: HashMap::new(),
        }],
        edges: vec![],
        resources: vec![resource("missing")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("has no producer"));
}

#[test]
fn rejects_consumer_before_producer() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            RenderGraphNode {
                node_id: id("consume"),
                pass_id: RENDER_PASS_FORWARD.into(),
                inputs: vec![id("r")],
                outputs: vec![],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: id("produce"),
                pass_id: RENDER_PASS_SHADOW.into(),
                inputs: vec![],
                outputs: vec![id("r")],
                params: HashMap::new(),
            },
        ],
        edges: vec![],
        resources: vec![resource("r")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("consumed before any producer"));
}

#[test]
fn rejects_multiple_writers_without_read_before_overwrite() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            RenderGraphNode {
                node_id: id("w0"),
                pass_id: RENDER_PASS_SHADOW.into(),
                inputs: vec![],
                outputs: vec![id("r")],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: id("w1"),
                pass_id: RENDER_PASS_POST.into(),
                inputs: vec![],
                outputs: vec![id("r")],
                params: HashMap::new(),
            },
        ],
        edges: vec![RenderGraphEdge {
            from_node_id: id("w0"),
            to_node_id: id("w1"),
            reason: None,
        }],
        resources: vec![resource("r")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("must read it before overwrite"));
}

#[test]
fn rejects_edge_reason_without_matching_dependency() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            RenderGraphNode {
                node_id: id("a"),
                pass_id: RENDER_PASS_LIGHT_CULL.into(),
                inputs: vec![],
                outputs: vec![],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: id("b"),
                pass_id: RENDER_PASS_SKYBOX.into(),
                inputs: vec![],
                outputs: vec![],
                params: HashMap::new(),
            },
        ],
        edges: vec![RenderGraphEdge {
            from_node_id: id("a"),
            to_node_id: id("b"),
            reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
        }],
        resources: vec![],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("no matching resource dependency"));
}
