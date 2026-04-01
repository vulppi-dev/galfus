use std::collections::HashMap;

use crate::{
    LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphLifetime,
    RenderGraphNode, RenderGraphResource, RenderGraphResourceKind,
};

pub fn ui_fallback_graph() -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str("ui_fallback".into()),
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str("ui_pass".into()),
            pass_id: "ui".into(),
            inputs: Vec::new(),
            outputs: vec![LogicalId::Str("swapchain".into())],
            params: HashMap::new(),
        }],
        edges: Vec::new(),
        resources: vec![RenderGraphResource {
            res_id: LogicalId::Str("swapchain".into()),
            kind: RenderGraphResourceKind::Attachment,
            lifetime: RenderGraphLifetime::Frame,
            alias_group: None,
        }],
        fallback: true,
    }
}

pub fn fallback_graph() -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str("fallback".into()),
        nodes: vec![
            RenderGraphNode {
                node_id: LogicalId::Str("shadow_pass".into()),
                pass_id: "shadow".into(),
                inputs: Vec::new(),
                outputs: vec![LogicalId::Str("shadow_atlas".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("light_cull_pass".into()),
                pass_id: "light-cull".into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("skybox_pass".into()),
                pass_id: "skybox".into(),
                inputs: Vec::new(),
                outputs: vec![LogicalId::Str("hdr_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("forward_pass".into()),
                pass_id: "forward".into(),
                inputs: vec![
                    LogicalId::Str("shadow_atlas".into()),
                    LogicalId::Str("hdr_color".into()),
                ],
                outputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("depth".into()),
                ],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("outline_pass".into()),
                pass_id: "outline".into(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("outline_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_pass".into()),
                pass_id: "ssao".into(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("ssao_raw".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_blur_pass".into()),
                pass_id: "ssao-blur".into(),
                inputs: vec![
                    LogicalId::Str("ssao_raw".into()),
                    LogicalId::Str("depth".into()),
                ],
                outputs: vec![LogicalId::Str("ssao_blur".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("bloom_pass".into()),
                pass_id: "bloom".into(),
                inputs: vec![LogicalId::Str("hdr_color".into())],
                outputs: vec![LogicalId::Str("bloom_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("post_pass".into()),
                pass_id: "post".into(),
                inputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("outline_color".into()),
                    LogicalId::Str("ssao_blur".into()),
                    LogicalId::Str("bloom_color".into()),
                ],
                outputs: vec![LogicalId::Str("post_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("compose_pass".into()),
                pass_id: "compose".into(),
                inputs: vec![LogicalId::Str("post_color".into())],
                outputs: vec![LogicalId::Str("swapchain".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ui_pass".into()),
                pass_id: "ui".into(),
                inputs: vec![LogicalId::Str("swapchain".into())],
                outputs: vec![LogicalId::Str("swapchain".into())],
                params: HashMap::new(),
            },
        ],
        edges: vec![
            RenderGraphEdge {
                from_node_id: LogicalId::Str("shadow_pass".into()),
                to_node_id: LogicalId::Str("forward_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("light_cull_pass".into()),
                to_node_id: LogicalId::Str("skybox_pass".into()),
                reason: None,
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("skybox_pass".into()),
                to_node_id: LogicalId::Str("forward_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("outline_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("ssao_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_pass".into()),
                to_node_id: LogicalId::Str("ssao_blur_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_blur_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("bloom_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("bloom_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("outline_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("post_pass".into()),
                to_node_id: LogicalId::Str("compose_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("compose_pass".into()),
                to_node_id: LogicalId::Str("ui_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
        ],
        resources: vec![
            RenderGraphResource {
                res_id: LogicalId::Str("shadow_atlas".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("hdr_color".into()),
                kind: RenderGraphResourceKind::Attachment,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("depth".into()),
                kind: RenderGraphResourceKind::Attachment,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("outline_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("ssao_raw".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("ssao_blur".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("bloom_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("post_color".into()),
                kind: RenderGraphResourceKind::Attachment,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("swapchain".into()),
                kind: RenderGraphResourceKind::Attachment,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
        ],
        fallback: true,
    }
}
