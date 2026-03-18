use std::collections::{HashMap, HashSet, VecDeque};

use super::{LogicalId, RenderGraphDesc, RenderGraphEdgeReason};

pub(super) fn validate_graph_semantics(
    desc: &RenderGraphDesc,
    order: &[usize],
) -> Result<(), String> {
    let mut node_index: HashMap<&LogicalId, usize> = HashMap::new();
    for (idx, node) in desc.nodes.iter().enumerate() {
        node_index.insert(&node.node_id, idx);
    }

    let mut topo_pos = vec![0usize; desc.nodes.len()];
    for (pos, node_idx) in order.iter().copied().enumerate() {
        topo_pos[node_idx] = pos;
    }

    let mut writers: HashMap<&LogicalId, Vec<usize>> = HashMap::new();
    for (node_idx, node) in desc.nodes.iter().enumerate() {
        for output in &node.outputs {
            writers.entry(output).or_default().push(node_idx);
        }
    }

    for (consumer_idx, node) in desc.nodes.iter().enumerate() {
        for input in &node.inputs {
            let Some(resource_writers) = writers.get(input) else {
                return Err(format!(
                    "Input resource '{}' in node '{}' has no producer",
                    input, node.node_id
                ));
            };
            let has_prior_writer = resource_writers
                .iter()
                .copied()
                .any(|writer_idx| topo_pos[writer_idx] < topo_pos[consumer_idx]);
            if !has_prior_writer {
                return Err(format!(
                    "Input resource '{}' in node '{}' is consumed before any producer",
                    input, node.node_id
                ));
            }
        }
    }

    let adjacency = build_adjacency(desc, &node_index)?;
    for (resource_id, resource_writers) in &writers {
        if resource_writers.len() <= 1 {
            continue;
        }
        let mut sorted_writers = resource_writers.clone();
        sorted_writers.sort_by_key(|node_idx| topo_pos[*node_idx]);
        for writer_pair in sorted_writers.windows(2) {
            let previous_writer = writer_pair[0];
            let next_writer = writer_pair[1];
            let next_node = &desc.nodes[next_writer];
            if !contains_id(&next_node.inputs, resource_id) {
                return Err(format!(
                    "Resource '{}' is written multiple times; node '{}' must read it before overwrite",
                    resource_id, next_node.node_id
                ));
            }
            if !has_path(&adjacency, previous_writer, next_writer) {
                return Err(format!(
                    "Resource '{}' is written multiple times but nodes '{}' -> '{}' have no dependency path",
                    resource_id, desc.nodes[previous_writer].node_id, next_node.node_id
                ));
            }
        }
    }

    for edge in &desc.edges {
        let Some(reason) = edge.reason else {
            continue;
        };
        let from_idx = *node_index
            .get(&edge.from_node_id)
            .ok_or_else(|| format!("Edge from unknown node: {}", edge.from_node_id))?;
        let to_idx = *node_index
            .get(&edge.to_node_id)
            .ok_or_else(|| format!("Edge to unknown node: {}", edge.to_node_id))?;
        let from_node = &desc.nodes[from_idx];
        let to_node = &desc.nodes[to_idx];
        let valid_reason = match reason {
            RenderGraphEdgeReason::ReadAfterWrite => {
                has_shared(&from_node.outputs, &to_node.inputs)
            }
            RenderGraphEdgeReason::WriteAfterRead => {
                has_shared(&from_node.inputs, &to_node.outputs)
            }
        };
        if !valid_reason {
            return Err(format!(
                "Edge '{}' -> '{}' has reason '{:?}' but no matching resource dependency",
                edge.from_node_id, edge.to_node_id, reason
            ));
        }
    }

    Ok(())
}

fn contains_id(items: &[LogicalId], needle: &LogicalId) -> bool {
    items.iter().any(|item| item == needle)
}

fn has_shared(left: &[LogicalId], right: &[LogicalId]) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }
    let left_set: HashSet<&LogicalId> = left.iter().collect();
    right.iter().any(|item| left_set.contains(item))
}

fn build_adjacency(
    desc: &RenderGraphDesc,
    node_index: &HashMap<&LogicalId, usize>,
) -> Result<Vec<Vec<usize>>, String> {
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); desc.nodes.len()];
    for edge in &desc.edges {
        let from_idx = *node_index
            .get(&edge.from_node_id)
            .ok_or_else(|| format!("Edge from unknown node: {}", edge.from_node_id))?;
        let to_idx = *node_index
            .get(&edge.to_node_id)
            .ok_or_else(|| format!("Edge to unknown node: {}", edge.to_node_id))?;
        adjacency[from_idx].push(to_idx);
    }
    Ok(adjacency)
}

fn has_path(adjacency: &[Vec<usize>], from: usize, to: usize) -> bool {
    if from == to {
        return true;
    }
    let mut visited = vec![false; adjacency.len()];
    let mut queue: VecDeque<usize> = VecDeque::new();
    queue.push_back(from);
    visited[from] = true;
    while let Some(current) = queue.pop_front() {
        for next in adjacency[current].iter().copied() {
            if next == to {
                return true;
            }
            if !visited[next] {
                visited[next] = true;
                queue.push_back(next);
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::{
        LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphNode,
        RenderGraphResource, RenderGraphResourceKind, RenderGraphState, validate_graph,
    };

    fn id(name: &str) -> LogicalId {
        LogicalId::Str(name.into())
    }

    fn resource(name: &str) -> RenderGraphResource {
        RenderGraphResource {
            res_id: id(name),
            kind: RenderGraphResourceKind::Texture,
            lifetime: super::super::RenderGraphLifetime::Frame,
            alias_group: None,
        }
    }

    #[test]
    fn fallback_graph_remains_semantically_valid() {
        let fallback = super::super::fallback_graph();
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
                pass_id: "forward".into(),
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
                    pass_id: "forward".into(),
                    inputs: vec![id("r")],
                    outputs: vec![],
                    params: HashMap::new(),
                },
                RenderGraphNode {
                    node_id: id("produce"),
                    pass_id: "shadow".into(),
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
                    pass_id: "shadow".into(),
                    inputs: vec![],
                    outputs: vec![id("r")],
                    params: HashMap::new(),
                },
                RenderGraphNode {
                    node_id: id("w1"),
                    pass_id: "post".into(),
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
                    pass_id: "light-cull".into(),
                    inputs: vec![],
                    outputs: vec![],
                    params: HashMap::new(),
                },
                RenderGraphNode {
                    node_id: id("b"),
                    pass_id: "skybox".into(),
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
}
