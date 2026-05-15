use std::collections::{HashMap, HashSet, VecDeque};

use crate::{LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphNode};

pub(super) fn derive_graph_edges(nodes: &[RenderGraphNode]) -> Vec<RenderGraphEdge> {
    let mut edges: Vec<RenderGraphEdge> = Vec::new();
    let mut writers: HashMap<&LogicalId, Vec<usize>> = HashMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        for output in &node.outputs {
            writers.entry(output).or_default().push(idx);
        }
    }

    for (consumer_idx, node) in nodes.iter().enumerate() {
        for input in &node.inputs {
            let Some(resource_writers) = writers.get(input) else {
                continue;
            };
            if let Some(writer_idx) = resource_writers.iter().copied().max() {
                if writer_idx != consumer_idx {
                    edges.push(RenderGraphEdge {
                        from_node_id: nodes[writer_idx].node_id.clone(),
                        to_node_id: node.node_id.clone(),
                        reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
                    });
                }
            }
        }
    }

    for output_writers in writers.values() {
        if output_writers.len() <= 1 {
            continue;
        }
        let mut sorted = output_writers.clone();
        sorted.sort_by_key(|idx| (nodes[*idx].priority, nodes[*idx].node_id.to_string()));
        for pair in sorted.windows(2) {
            let from = pair[0];
            let to = pair[1];
            edges.push(RenderGraphEdge {
                from_node_id: nodes[from].node_id.clone(),
                to_node_id: nodes[to].node_id.clone(),
                reason: Some(RenderGraphEdgeReason::WriteAfterRead),
            });
        }
    }

    for (consumer_idx, node) in nodes.iter().enumerate() {
        for required in &node.require {
            let Some(resource_writers) = writers.get(required) else {
                continue;
            };
            if let Some(writer_idx) = resource_writers.iter().copied().max() {
                if writer_idx != consumer_idx {
                    edges.push(RenderGraphEdge {
                        from_node_id: nodes[writer_idx].node_id.clone(),
                        to_node_id: node.node_id.clone(),
                        reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
                    });
                }
            }
        }
    }

    edges
}

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
        for required in &node.require {
            let Some(resource_writers) = writers.get(required) else {
                return Err(format!(
                    "Required resource '{}' in node '{}' has no producer",
                    required, node.node_id
                ));
            };
            let has_prior_writer = resource_writers
                .iter()
                .copied()
                .any(|writer_idx| topo_pos[writer_idx] < topo_pos[consumer_idx]);
            if !has_prior_writer {
                return Err(format!(
                    "Required resource '{}' in node '{}' is consumed before any producer",
                    required, node.node_id
                ));
            }
        }
        for input in &node.inputs {
            let Some(resource_writers) = writers.get(input) else {
                continue;
            };
            let has_prior_writer = resource_writers
                .iter()
                .copied()
                .any(|writer_idx| topo_pos[writer_idx] < topo_pos[consumer_idx]);
            if !has_prior_writer {
                continue;
            }
        }
    }

    let adjacency = build_adjacency(desc, &node_index)?;
    for (resource_id, resource_writers) in &writers {
        if resource_writers.len() <= 1 {
            continue;
        }
        let mut read_write_nodes: Vec<usize> = resource_writers
            .iter()
            .copied()
            .filter(|node_idx| contains_id(&desc.nodes[*node_idx].inputs, resource_id))
            .collect();
        if read_write_nodes.len() > 1 {
            read_write_nodes.sort_by_key(|idx| desc.nodes[*idx].priority);
            for pair in read_write_nodes.windows(2) {
                let a = pair[0];
                let b = pair[1];
                if desc.nodes[a].priority == desc.nodes[b].priority {
                    return Err(format!(
                        "Resource '{}' has dangerous same-priority overwrite between '{}' and '{}'; set explicit priority",
                        resource_id, desc.nodes[a].node_id, desc.nodes[b].node_id
                    ));
                }
            }
        }
        let mut sorted_writers = resource_writers.clone();
        sorted_writers.sort_by_key(|node_idx| (topo_pos[*node_idx], desc.nodes[*node_idx].priority));
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
#[path = "validation_tests.rs"]
mod tests;
