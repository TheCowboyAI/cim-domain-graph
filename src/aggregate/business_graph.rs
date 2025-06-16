//! Business Graph Aggregate
//!
//! Represents a runtime graph structure with nodes and edges that can be
//! manipulated through commands and used for workflow or data modeling.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use cim_domain::AggregateRoot;
use crate::{GraphId, NodeId, EdgeId};
use crate::commands::GraphCommandError;

/// Business node in a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node
    pub id: NodeId,
    /// Type/category of the node (e.g., "task", "decision", "gateway")
    pub node_type: String,
    /// Additional metadata about the node
    pub metadata: HashMap<String, serde_json::Value>,
}

impl GraphNode {
    /// Create a new graph node
    pub fn new(id: NodeId, node_type: String, metadata: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id,
            node_type,
            metadata,
        }
    }
}

/// Business edge in a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Unique identifier for the edge
    pub id: EdgeId,
    /// Source node of the edge
    pub source_id: NodeId,
    /// Target node of the edge
    pub target_id: NodeId,
    /// Type/category of the edge (e.g., "sequence", "conditional", "parallel")
    pub edge_type: String,
    /// Additional metadata about the edge
    pub metadata: HashMap<String, serde_json::Value>,
}

impl GraphEdge {
    /// Create a new graph edge
    pub fn new(
        id: EdgeId,
        source_id: NodeId,
        target_id: NodeId,
        edge_type: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id,
            source_id,
            target_id,
            edge_type,
            metadata,
        }
    }
}

/// Business Graph aggregate for runtime operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Unique identifier for the graph
    id: GraphId,
    /// Human-readable name of the graph
    name: String,
    /// Description of the graph's purpose
    description: String,
    /// All nodes in the graph
    nodes: HashMap<NodeId, GraphNode>,
    /// All edges in the graph
    edges: HashMap<EdgeId, GraphEdge>,
    /// Additional metadata about the graph
    metadata: HashMap<String, serde_json::Value>,
    /// When the graph was created
    created_at: chrono::DateTime<chrono::Utc>,
    /// When the graph was last modified
    last_modified: chrono::DateTime<chrono::Utc>,
    /// Version for optimistic concurrency control
    version: u64,
}

impl Graph {
    /// Create a new graph
    pub fn new(id: GraphId, name: String, description: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            description,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            metadata: HashMap::new(),
            created_at: now,
            last_modified: now,
            version: 1,
        }
    }

    /// Get the graph's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the graph's description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get all nodes
    pub fn nodes(&self) -> &HashMap<NodeId, GraphNode> {
        &self.nodes
    }

    /// Get all edges
    pub fn edges(&self) -> &HashMap<EdgeId, GraphEdge> {
        &self.edges
    }

    /// Get graph metadata
    pub fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// Get last modified timestamp
    pub fn last_modified(&self) -> chrono::DateTime<chrono::Utc> {
        self.last_modified
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Add a node to the graph
    pub fn add_node(
        &mut self,
        node_id: NodeId,
        node_type: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), GraphCommandError> {
        // Check if node already exists
        if self.nodes.contains_key(&node_id) {
            return Err(GraphCommandError::BusinessRuleViolation(
                format!("Node {} already exists", node_id)
            ));
        }

        // Create and add the node
        let node = GraphNode::new(node_id, node_type, metadata);
        self.nodes.insert(node_id, node);
        self.last_modified = chrono::Utc::now();
        self.version += 1;

        Ok(())
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), GraphCommandError> {
        // Check if node exists
        if !self.nodes.contains_key(&node_id) {
            return Err(GraphCommandError::NodeNotFound(node_id));
        }

        // Remove all edges connected to this node
        let connected_edges: Vec<EdgeId> = self.edges
            .values()
            .filter(|edge| edge.source_id == node_id || edge.target_id == node_id)
            .map(|edge| edge.id)
            .collect();

        for edge_id in connected_edges {
            self.edges.remove(&edge_id);
        }

        // Remove the node
        self.nodes.remove(&node_id);
        self.last_modified = chrono::Utc::now();
        self.version += 1;

        Ok(())
    }

    /// Change a node's metadata by removing old and adding new
    pub fn change_node_metadata(
        &mut self,
        node_id: NodeId,
        new_metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), GraphCommandError> {
        // Check if node exists
        if !self.nodes.contains_key(&node_id) {
            return Err(GraphCommandError::NodeNotFound(node_id));
        }

        // Remove the old node
        let old_node = self.nodes.remove(&node_id).unwrap();
        
        // Create new node with updated metadata
        let new_node = GraphNode::new(node_id, old_node.node_type, new_metadata);
        self.nodes.insert(node_id, new_node);
        
        self.last_modified = chrono::Utc::now();
        self.version += 1;

        Ok(())
    }

    /// Add an edge to the graph
    pub fn add_edge(
        &mut self,
        edge_id: EdgeId,
        source_id: NodeId,
        target_id: NodeId,
        edge_type: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), GraphCommandError> {
        // Check if edge already exists
        if self.edges.contains_key(&edge_id) {
            return Err(GraphCommandError::BusinessRuleViolation(
                format!("Edge {} already exists", edge_id)
            ));
        }

        // Check if source and target nodes exist
        if !self.nodes.contains_key(&source_id) {
            return Err(GraphCommandError::NodeNotFound(source_id));
        }
        if !self.nodes.contains_key(&target_id) {
            return Err(GraphCommandError::NodeNotFound(target_id));
        }

        // Create and add the edge
        let edge = GraphEdge::new(edge_id, source_id, target_id, edge_type, metadata);
        self.edges.insert(edge_id, edge);
        self.last_modified = chrono::Utc::now();
        self.version += 1;

        Ok(())
    }

    /// Remove an edge from the graph
    pub fn remove_edge(&mut self, edge_id: EdgeId) -> Result<(), GraphCommandError> {
        // Check if edge exists
        if !self.edges.contains_key(&edge_id) {
            return Err(GraphCommandError::EdgeNotFound(edge_id));
        }

        // Remove the edge
        self.edges.remove(&edge_id);
        self.last_modified = chrono::Utc::now();
        self.version += 1;

        Ok(())
    }

    /// Get incoming edges for a node
    pub fn get_incoming_edges(&self, node_id: NodeId) -> Vec<&GraphEdge> {
        self.edges
            .values()
            .filter(|edge| edge.target_id == node_id)
            .collect()
    }

    /// Get outgoing edges for a node
    pub fn get_outgoing_edges(&self, node_id: NodeId) -> Vec<&GraphEdge> {
        self.edges
            .values()
            .filter(|edge| edge.source_id == node_id)
            .collect()
    }

    /// Check if the graph contains cycles (simple DFS-based detection)
    pub fn has_cycles(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if self.has_cycle_util(*node_id, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// Utility function for cycle detection
    fn has_cycle_util(
        &self,
        node_id: NodeId,
        visited: &mut std::collections::HashSet<NodeId>,
        rec_stack: &mut std::collections::HashSet<NodeId>,
    ) -> bool {
        visited.insert(node_id);
        rec_stack.insert(node_id);

        // Check all adjacent nodes
        for edge in self.get_outgoing_edges(node_id) {
            if !visited.contains(&edge.target_id) {
                if self.has_cycle_util(edge.target_id, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&edge.target_id) {
                return true;
            }
        }

        rec_stack.remove(&node_id);
        false
    }

    /// Get source nodes (nodes with no incoming edges)
    pub fn get_source_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .keys()
            .filter(|node_id| self.get_incoming_edges(**node_id).is_empty())
            .copied()
            .collect()
    }

    /// Get sink nodes (nodes with no outgoing edges)
    pub fn get_sink_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .keys()
            .filter(|node_id| self.get_outgoing_edges(**node_id).is_empty())
            .copied()
            .collect()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl AggregateRoot for Graph {
    type Id = GraphId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Coverage
    ///
    /// ```mermaid
    /// graph TD
    ///     G[Graph Aggregate] --> N[Node Operations]
    ///     G --> E[Edge Operations]
    ///     G --> V[Validation]
    ///     G --> C[Cycle Detection]
    /// ```

    #[test]
    fn test_graph_creation() {
        let graph_id = GraphId::new();
        let graph = Graph::new(
            graph_id,
            "Test Graph".to_string(),
            "A test graph".to_string(),
        );

        assert_eq!(graph.id(), graph_id.into());
        assert_eq!(graph.name(), "Test Graph");
        assert_eq!(graph.description(), "A test graph");
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.version(), 1);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new(
            GraphId::new(),
            "Test Graph".to_string(),
            "A test graph".to_string(),
        );

        let node_id = NodeId::new();
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), serde_json::Value::String("Test Node".to_string()));

        let result = graph.add_node(node_id, "task".to_string(), metadata);
        assert!(result.is_ok());
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.version(), 2);

        // Test duplicate node
        let duplicate_result = graph.add_node(node_id, "task".to_string(), HashMap::new());
        assert!(duplicate_result.is_err());
    }

    #[test]
    fn test_remove_node() {
        let mut graph = Graph::new(
            GraphId::new(),
            "Test Graph".to_string(),
            "A test graph".to_string(),
        );

        let node_id = NodeId::new();
        graph.add_node(node_id, "task".to_string(), HashMap::new()).unwrap();

        let result = graph.remove_node(node_id);
        assert!(result.is_ok());
        assert_eq!(graph.node_count(), 0);

        // Test removing non-existent node
        let missing_result = graph.remove_node(NodeId::new());
        assert!(missing_result.is_err());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new(
            GraphId::new(),
            "Test Graph".to_string(),
            "A test graph".to_string(),
        );

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        graph.add_node(node1, "start".to_string(), HashMap::new()).unwrap();
        graph.add_node(node2, "end".to_string(), HashMap::new()).unwrap();

        let edge_id = EdgeId::new();
        let result = graph.add_edge(edge_id, node1, node2, "sequence".to_string(), HashMap::new());
        assert!(result.is_ok());
        assert_eq!(graph.edge_count(), 1);

        // Test edge to non-existent node
        let invalid_result = graph.add_edge(
            EdgeId::new(),
            node1,
            NodeId::new(),
            "sequence".to_string(),
            HashMap::new(),
        );
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = Graph::new(
            GraphId::new(),
            "Test Graph".to_string(),
            "A test graph".to_string(),
        );

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        graph.add_node(node1, "task".to_string(), HashMap::new()).unwrap();
        graph.add_node(node2, "task".to_string(), HashMap::new()).unwrap();
        graph.add_node(node3, "task".to_string(), HashMap::new()).unwrap();

        // Create a cycle: node1 -> node2 -> node3 -> node1
        graph.add_edge(EdgeId::new(), node1, node2, "sequence".to_string(), HashMap::new()).unwrap();
        graph.add_edge(EdgeId::new(), node2, node3, "sequence".to_string(), HashMap::new()).unwrap();
        graph.add_edge(EdgeId::new(), node3, node1, "sequence".to_string(), HashMap::new()).unwrap();

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_source_and_sink_nodes() {
        let mut graph = Graph::new(
            GraphId::new(),
            "Test Graph".to_string(),
            "A test graph".to_string(),
        );

        let start_node = NodeId::new();
        let middle_node = NodeId::new();
        let end_node = NodeId::new();

        graph.add_node(start_node, "start".to_string(), HashMap::new()).unwrap();
        graph.add_node(middle_node, "task".to_string(), HashMap::new()).unwrap();
        graph.add_node(end_node, "end".to_string(), HashMap::new()).unwrap();

        graph.add_edge(EdgeId::new(), start_node, middle_node, "sequence".to_string(), HashMap::new()).unwrap();
        graph.add_edge(EdgeId::new(), middle_node, end_node, "sequence".to_string(), HashMap::new()).unwrap();

        let sources = graph.get_source_nodes();
        let sinks = graph.get_sink_nodes();

        assert_eq!(sources.len(), 1);
        assert!(sources.contains(&start_node));

        assert_eq!(sinks.len(), 1);
        assert!(sinks.contains(&end_node));
    }
} 