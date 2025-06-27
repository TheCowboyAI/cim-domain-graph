//! Graph abstraction layer that provides a unified interface for working with
//! any graph implementation (ContextGraph, ConceptGraph, WorkflowGraph, etc.)

use cim_domain::{NodeId, EdgeId, GraphId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod adapters;

// Re-export Position3D for convenience
pub use crate::value_objects::Position3D;

// Re-export adapters for convenience
pub use adapters::{
    ContextGraphAdapter, ConceptGraphAdapter, WorkflowGraphAdapter, IpldGraphAdapter,
};

/// Unified node data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub node_type: String,
    pub position: Position3D,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Unified edge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub edge_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Errors that can occur in graph operations
#[derive(Debug, thiserror::Error)]
pub enum GraphOperationError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    
    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),
    
    #[error("Invalid node type: {0}")]
    InvalidNodeType(String),
    
    #[error("Invalid edge type: {0}")]
    InvalidEdgeType(String),
    
    #[error("Graph creation failed: {0}")]
    GraphCreationFailed(String),
    
    #[error("Node creation failed: {0}")]
    NodeCreationFailed(String),
    
    #[error("Edge creation failed: {0}")]
    EdgeCreationFailed(String),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

/// Result type for graph operations
pub type GraphResult<T> = Result<T, GraphOperationError>;

/// Trait that all graph implementations must implement through adapters
pub trait GraphImplementation: Send + Sync {
    /// Get the graph ID
    fn graph_id(&self) -> GraphId;
    
    /// Add a node to the graph
    fn add_node(&mut self, node_id: NodeId, data: NodeData) -> GraphResult<()>;
    
    /// Add an edge to the graph
    fn add_edge(&mut self, edge_id: EdgeId, source: NodeId, target: NodeId, data: EdgeData) -> GraphResult<()>;
    
    /// Get a node by ID
    fn get_node(&self, node_id: NodeId) -> GraphResult<NodeData>;
    
    /// Get an edge by ID
    fn get_edge(&self, edge_id: EdgeId) -> GraphResult<(EdgeData, NodeId, NodeId)>;
    
    /// List all nodes
    fn list_nodes(&self) -> Vec<(NodeId, NodeData)>;
    
    /// List all edges
    fn list_edges(&self) -> Vec<(EdgeId, EdgeData, NodeId, NodeId)>;
    
    /// Get graph metadata
    fn get_metadata(&self) -> GraphMetadata;
    
    /// Update graph metadata
    fn update_metadata(&mut self, metadata: GraphMetadata) -> GraphResult<()>;
    
    /// Find nodes by type
    fn find_nodes_by_type(&self, node_type: &str) -> Vec<NodeId>;
    
    /// Find edges by type
    fn find_edges_by_type(&self, edge_type: &str) -> Vec<EdgeId>;
}

/// Enum of available graph types with their implementations
#[derive(Clone)]
pub enum GraphType {
    Context(ContextGraphAdapter),
    Concept(ConceptGraphAdapter),
    Workflow(WorkflowGraphAdapter),
    Ipld(IpldGraphAdapter),
}

impl GraphType {
    /// Create a new context graph
    pub fn new_context(graph_id: GraphId, name: &str) -> Self {
        GraphType::Context(ContextGraphAdapter::new(graph_id, name.to_string()))
    }
    
    /// Create a new concept graph
    pub fn new_concept(graph_id: GraphId, name: &str) -> Self {
        GraphType::Concept(ConceptGraphAdapter::new(graph_id, name))
    }
    
    /// Create a new workflow graph
    pub fn new_workflow(graph_id: GraphId, name: &str) -> Self {
        GraphType::Workflow(WorkflowGraphAdapter::new(graph_id, name))
    }
    
    /// Create a new IPLD graph
    pub fn new_ipld(graph_id: GraphId) -> Self {
        GraphType::Ipld(IpldGraphAdapter::new(graph_id))
    }
}

// Implement GraphImplementation for GraphType by delegating to the inner implementation
impl GraphImplementation for GraphType {
    fn graph_id(&self) -> GraphId {
        match self {
            GraphType::Context(adapter) => adapter.graph_id(),
            GraphType::Concept(adapter) => adapter.graph_id(),
            GraphType::Workflow(adapter) => adapter.graph_id(),
            GraphType::Ipld(adapter) => adapter.graph_id(),
        }
    }
    
    fn add_node(&mut self, node_id: NodeId, data: NodeData) -> GraphResult<()> {
        match self {
            GraphType::Context(adapter) => adapter.add_node(node_id, data),
            GraphType::Concept(adapter) => adapter.add_node(node_id, data),
            GraphType::Workflow(adapter) => adapter.add_node(node_id, data),
            GraphType::Ipld(adapter) => adapter.add_node(node_id, data),
        }
    }
    
    fn add_edge(&mut self, edge_id: EdgeId, source: NodeId, target: NodeId, data: EdgeData) -> GraphResult<()> {
        match self {
            GraphType::Context(adapter) => adapter.add_edge(edge_id, source, target, data),
            GraphType::Concept(adapter) => adapter.add_edge(edge_id, source, target, data),
            GraphType::Workflow(adapter) => adapter.add_edge(edge_id, source, target, data),
            GraphType::Ipld(adapter) => adapter.add_edge(edge_id, source, target, data),
        }
    }
    
    fn get_node(&self, node_id: NodeId) -> GraphResult<NodeData> {
        match self {
            GraphType::Context(adapter) => adapter.get_node(node_id),
            GraphType::Concept(adapter) => adapter.get_node(node_id),
            GraphType::Workflow(adapter) => adapter.get_node(node_id),
            GraphType::Ipld(adapter) => adapter.get_node(node_id),
        }
    }
    
    fn get_edge(&self, edge_id: EdgeId) -> GraphResult<(EdgeData, NodeId, NodeId)> {
        match self {
            GraphType::Context(adapter) => adapter.get_edge(edge_id),
            GraphType::Concept(adapter) => adapter.get_edge(edge_id),
            GraphType::Workflow(adapter) => adapter.get_edge(edge_id),
            GraphType::Ipld(adapter) => adapter.get_edge(edge_id),
        }
    }
    
    fn list_nodes(&self) -> Vec<(NodeId, NodeData)> {
        match self {
            GraphType::Context(adapter) => adapter.list_nodes(),
            GraphType::Concept(adapter) => adapter.list_nodes(),
            GraphType::Workflow(adapter) => adapter.list_nodes(),
            GraphType::Ipld(adapter) => adapter.list_nodes(),
        }
    }
    
    fn list_edges(&self) -> Vec<(EdgeId, EdgeData, NodeId, NodeId)> {
        match self {
            GraphType::Context(adapter) => adapter.list_edges(),
            GraphType::Concept(adapter) => adapter.list_edges(),
            GraphType::Workflow(adapter) => adapter.list_edges(),
            GraphType::Ipld(adapter) => adapter.list_edges(),
        }
    }
    
    fn get_metadata(&self) -> GraphMetadata {
        match self {
            GraphType::Context(adapter) => adapter.get_metadata(),
            GraphType::Concept(adapter) => adapter.get_metadata(),
            GraphType::Workflow(adapter) => adapter.get_metadata(),
            GraphType::Ipld(adapter) => adapter.get_metadata(),
        }
    }
    
    fn update_metadata(&mut self, metadata: GraphMetadata) -> GraphResult<()> {
        match self {
            GraphType::Context(adapter) => adapter.update_metadata(metadata),
            GraphType::Concept(adapter) => adapter.update_metadata(metadata),
            GraphType::Workflow(adapter) => adapter.update_metadata(metadata),
            GraphType::Ipld(adapter) => adapter.update_metadata(metadata),
        }
    }
    
    fn find_nodes_by_type(&self, node_type: &str) -> Vec<NodeId> {
        match self {
            GraphType::Context(adapter) => adapter.find_nodes_by_type(node_type),
            GraphType::Concept(adapter) => adapter.find_nodes_by_type(node_type),
            GraphType::Workflow(adapter) => adapter.find_nodes_by_type(node_type),
            GraphType::Ipld(adapter) => adapter.find_nodes_by_type(node_type),
        }
    }
    
    fn find_edges_by_type(&self, edge_type: &str) -> Vec<EdgeId> {
        match self {
            GraphType::Context(adapter) => adapter.find_edges_by_type(edge_type),
            GraphType::Concept(adapter) => adapter.find_edges_by_type(edge_type),
            GraphType::Workflow(adapter) => adapter.find_edges_by_type(edge_type),
            GraphType::Ipld(adapter) => adapter.find_edges_by_type(edge_type),
        }
    }
} 