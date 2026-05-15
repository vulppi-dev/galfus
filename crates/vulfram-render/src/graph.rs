use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use vulfram_realm_core::RealmKind;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LogicalId {
    Str(String),
    Int(i64),
}

impl std::fmt::Display for LogicalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalId::Str(value) => write!(f, "{}", value),
            LogicalId::Int(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphResourceKind {
    Texture,
    Buffer,
    Attachment,
}

impl Default for RenderGraphResourceKind {
    fn default() -> Self {
        Self::Texture
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphLifetime {
    Frame,
    Persistent,
}

impl Default for RenderGraphLifetime {
    fn default() -> Self {
        Self::Frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphEdgeReason {
    ReadAfterWrite,
    WriteAfterRead,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RenderGraphValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl From<&str> for RenderGraphValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for RenderGraphValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for RenderGraphValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for RenderGraphValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for RenderGraphValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphResource {
    pub res_id: LogicalId,
    #[serde(default)]
    pub kind: RenderGraphResourceKind,
    #[serde(default)]
    pub lifetime: RenderGraphLifetime,
    #[serde(default)]
    pub alias_group: Option<LogicalId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphNode {
    pub node_id: LogicalId,
    pub pass_id: String,
    #[serde(default)]
    pub inputs: Vec<LogicalId>,
    #[serde(default)]
    pub outputs: Vec<LogicalId>,
    #[serde(default)]
    pub require: Vec<LogicalId>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_graph_node_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub params: HashMap<String, RenderGraphValue>,
}

fn default_graph_node_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphEdge {
    pub from_node_id: LogicalId,
    pub to_node_id: LogicalId,
    #[serde(default)]
    pub reason: Option<RenderGraphEdgeReason>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphDesc {
    pub graph_id: LogicalId,
    pub nodes: Vec<RenderGraphNode>,
    pub edges: Vec<RenderGraphEdge>,
    #[serde(default)]
    pub resources: Vec<RenderGraphResource>,
    #[serde(default)]
    pub fallback: bool,
}

#[derive(Debug, Clone)]
pub struct RenderGraphPlan {
    pub nodes: Vec<RenderGraphNode>,
    pub order: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct RenderGraphState {
    active: RenderGraphPlan,
}

impl Default for RenderGraphState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RenderGraphRecord {
    pub state: RenderGraphState,
    pub desc_hash: u64,
}

impl RenderGraphState {
    pub fn new() -> Self {
        Self::new_with_fallback(crate::fallback_graph())
    }

    pub fn new_ui() -> Self {
        Self::new_with_fallback(crate::ui_fallback_graph())
    }

    pub fn new_with_fallback(fallback_desc: RenderGraphDesc) -> Self {
        let fallback = validate_graph(&fallback_desc).expect("Fallback graph must be valid");
        Self { active: fallback }
    }

    pub fn plan(&self) -> &RenderGraphPlan {
        &self.active
    }

    pub fn from_desc(desc: RenderGraphDesc) -> Result<Self, String> {
        let active = validate_graph(&desc)?;
        Ok(Self { active })
    }
}

pub const DEFAULT_3D_RENDER_GRAPH_ID: u32 = 1;
pub const DEFAULT_2D_RENDER_GRAPH_ID: u32 = 2;

pub fn fallback_render_graph_id(realm_kind: RealmKind) -> u32 {
    match realm_kind {
        RealmKind::ThreeD => DEFAULT_3D_RENDER_GRAPH_ID,
        RealmKind::TwoD => DEFAULT_2D_RENDER_GRAPH_ID,
    }
}

pub fn resolve_render_graph_id(
    explicit_render_graph_id: Option<u32>,
    realm_kind: RealmKind,
) -> u32 {
    explicit_render_graph_id.unwrap_or_else(|| fallback_render_graph_id(realm_kind))
}

pub fn is_reserved_render_graph_id(render_graph_id: u32) -> bool {
    render_graph_id == DEFAULT_3D_RENDER_GRAPH_ID || render_graph_id == DEFAULT_2D_RENDER_GRAPH_ID
}

pub fn render_graph_desc_hash(desc: &RenderGraphDesc) -> u64 {
    let mut hasher = DefaultHasher::new();
    match rmp_serde::to_vec_named(desc) {
        Ok(bytes) => bytes.hash(&mut hasher),
        Err(_) => format!("{:?}", desc.graph_id).hash(&mut hasher),
    }
    hasher.finish()
}

pub fn ensure_default_render_graphs(
    graphs: &mut HashMap<u32, RenderGraphRecord>,
    cache: &mut HashMap<u64, RenderGraphState>,
) {
    let fallback_3d = crate::fallback_graph();
    let hash_3d = render_graph_desc_hash(&fallback_3d);
    let state_3d = cache.entry(hash_3d).or_default().clone();
    graphs
        .entry(DEFAULT_3D_RENDER_GRAPH_ID)
        .or_insert(RenderGraphRecord {
            state: state_3d,
            desc_hash: hash_3d,
        });

    let fallback_2d = crate::ui_fallback_graph();
    let hash_2d = render_graph_desc_hash(&fallback_2d);
    let state_2d = cache
        .entry(hash_2d)
        .or_insert_with(RenderGraphState::new_ui)
        .clone();
    graphs
        .entry(DEFAULT_2D_RENDER_GRAPH_ID)
        .or_insert(RenderGraphRecord {
            state: state_2d,
            desc_hash: hash_2d,
        });
}

pub fn validate_graph(desc: &RenderGraphDesc) -> Result<RenderGraphPlan, String> {
    let mut node_ids: HashSet<LogicalId> = HashSet::new();
    let enabled_nodes: Vec<_> = desc.nodes.iter().filter(|node| node.enabled).collect();
    for node in &enabled_nodes {
        if !node_ids.insert(node.node_id.clone()) {
            return Err(format!("Duplicate node_id: {}", node.node_id));
        }
    }

    let mut res_ids: HashSet<LogicalId> = HashSet::new();
    for res in &desc.resources {
        if !res_ids.insert(res.res_id.clone()) {
            return Err(format!("Duplicate res_id: {}", res.res_id));
        }
    }

    let mut node_index: HashMap<LogicalId, usize> = HashMap::new();
    let filtered_nodes: Vec<_> = enabled_nodes.into_iter().cloned().collect();
    for (idx, node) in filtered_nodes.iter().enumerate() {
        node_index.insert(node.node_id.clone(), idx);
        if !is_known_pass(&node.pass_id) {
            return Err(format!("Unknown pass_id: {}", node.pass_id));
        }
    }

    for edge in &desc.edges {
        if !node_index.contains_key(&edge.from_node_id) || !node_index.contains_key(&edge.to_node_id) {
            continue;
        }
        if !node_index.contains_key(&edge.from_node_id) {
            return Err(format!("Edge from unknown node: {}", edge.from_node_id));
        }
        if !node_index.contains_key(&edge.to_node_id) {
            return Err(format!("Edge to unknown node: {}", edge.to_node_id));
        }
    }

    for node in &filtered_nodes {
        let mut node_inputs: HashSet<&LogicalId> = HashSet::new();
        for input in &node.inputs {
            if !node_inputs.insert(input) {
                return Err(format!(
                    "Duplicate input '{}' in node '{}'",
                    input, node.node_id
                ));
            }
            if !res_ids.contains(input) {
                return Err(format!("Input resource '{}' not declared", input));
            }
        }
        let mut node_require: HashSet<&LogicalId> = HashSet::new();
        for required in &node.require {
            if !node_require.insert(required) {
                return Err(format!(
                    "Duplicate require '{}' in node '{}'",
                    required, node.node_id
                ));
            }
            if !res_ids.contains(required) {
                return Err(format!("Required resource '{}' not declared", required));
            }
        }
        let mut node_outputs: HashSet<&LogicalId> = HashSet::new();
        for output in &node.outputs {
            if !node_outputs.insert(output) {
                return Err(format!(
                    "Duplicate output '{}' in node '{}'",
                    output, node.node_id
                ));
            }
            if !res_ids.contains(output) {
                return Err(format!("Output resource '{}' not declared", output));
            }
        }
    }

    let derived_edges = crate::validation::derive_graph_edges(&filtered_nodes);
    let mut all_edges: Vec<RenderGraphEdge> = Vec::new();
    for edge in &desc.edges {
        if node_index.contains_key(&edge.from_node_id) && node_index.contains_key(&edge.to_node_id) {
            all_edges.push(edge.clone());
        }
    }
    all_edges.extend(derived_edges);
    dedupe_edges(&mut all_edges);

    let filtered_desc = RenderGraphDesc {
        graph_id: desc.graph_id.clone(),
        nodes: filtered_nodes.clone(),
        edges: all_edges,
        resources: desc.resources.clone(),
        fallback: desc.fallback,
    };
    let order = topo_sort(&filtered_desc.nodes, &filtered_desc.edges)?;
    crate::validation::validate_graph_semantics(&filtered_desc, &order)?;

    Ok(RenderGraphPlan {
        nodes: filtered_nodes,
        order,
    })
}

fn dedupe_edges(edges: &mut Vec<RenderGraphEdge>) {
    edges.sort_by(|a, b| {
        let ak = (a.from_node_id.to_string(), a.to_node_id.to_string());
        let bk = (b.from_node_id.to_string(), b.to_node_id.to_string());
        ak.cmp(&bk)
    });
    edges.dedup_by(|a, b| a.from_node_id == b.from_node_id && a.to_node_id == b.to_node_id);
}

fn topo_sort(nodes: &[RenderGraphNode], edges: &[RenderGraphEdge]) -> Result<Vec<usize>, String> {
    let mut indegree = vec![0usize; nodes.len()];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    let mut index_map: HashMap<LogicalId, usize> = HashMap::new();

    for (idx, node) in nodes.iter().enumerate() {
        index_map.insert(node.node_id.clone(), idx);
    }

    for edge in edges {
        let from = *index_map
            .get(&edge.from_node_id)
            .ok_or_else(|| format!("Edge from unknown node: {}", edge.from_node_id))?;
        let to = *index_map
            .get(&edge.to_node_id)
            .ok_or_else(|| format!("Edge to unknown node: {}", edge.to_node_id))?;
        adjacency[from].push(to);
        indegree[to] += 1;
    }

    let mut queue = VecDeque::new();
    for (idx, &deg) in indegree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(idx);
        }
    }

    let mut order = Vec::with_capacity(nodes.len());
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &next in &adjacency[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                queue.push_back(next);
            }
        }
    }

    if order.len() != nodes.len() {
        return Err("Render graph contains a cycle".into());
    }

    Ok(order)
}

fn is_known_pass(pass_id: &str) -> bool {
    crate::supports_render_pass_for_realm_kind(pass_id, RealmKind::ThreeD)
        || crate::supports_render_pass_for_realm_kind(pass_id, RealmKind::TwoD)
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
