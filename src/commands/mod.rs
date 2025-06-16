//! Graph commands
//!
//! Commands represent intent to modify graph state. They are processed by command handlers
//! which validate business rules and emit corresponding events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{GraphId, NodeId, EdgeId};

/// Commands for graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Create a new graph
    CreateGraph {
        /// The name of the graph
        name: String,
        /// A description of the graph's purpose
        description: String,
        /// Additional metadata about the graph
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Add a node to a graph
    AddNode {
        /// The graph to add the node to
        graph_id: GraphId,
        /// The type of node (e.g., "task", "decision", "gateway")
        node_type: String,
        /// Additional metadata about the node
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Remove a node from a graph
    RemoveNode {
        /// The graph to remove the node from
        graph_id: GraphId,
        /// The ID of the node to remove
        node_id: NodeId,
    },
    
    /// Change a node's metadata by removing and re-adding
    ChangeNodeMetadata {
        /// The graph containing the node
        graph_id: GraphId,
        /// The ID of the node to change
        node_id: NodeId,
        /// The new metadata for the node (replaces all existing metadata)
        new_metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Add an edge to connect two nodes
    AddEdge {
        /// The graph to add the edge to
        graph_id: GraphId,
        /// The source node of the edge
        source_id: NodeId,
        /// The target node of the edge
        target_id: NodeId,
        /// The type of edge (e.g., "sequence", "conditional", "parallel")
        edge_type: String,
        /// Additional metadata about the edge
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Remove an edge from a graph
    RemoveEdge {
        /// The graph to remove the edge from
        graph_id: GraphId,
        /// The ID of the edge to remove
        edge_id: EdgeId,
    },
}

/// Commands for node operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeCommand {
    /// Add a node to a graph
    Add {
        /// The graph to add the node to
        graph_id: GraphId,
        /// The type of node
        node_type: String,
        /// Additional metadata about the node
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Remove a node from a graph
    Remove {
        /// The graph to remove the node from
        graph_id: GraphId,
        /// The ID of the node to remove
        node_id: NodeId,
    },
    
    /// Change a node's metadata by removing and re-adding
    ChangeMetadata {
        /// The graph containing the node
        graph_id: GraphId,
        /// The ID of the node to change
        node_id: NodeId,
        /// The new metadata for the node (replaces all existing metadata)
        new_metadata: HashMap<String, serde_json::Value>,
    },
}

/// Commands for edge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCommand {
    /// Add an edge between two nodes
    Add {
        /// The graph to add the edge to
        graph_id: GraphId,
        /// The source node of the edge
        source_id: NodeId,
        /// The target node of the edge
        target_id: NodeId,
        /// The type of edge
        edge_type: String,
        /// Additional metadata about the edge
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Remove an edge from a graph
    Remove {
        /// The graph to remove the edge from
        graph_id: GraphId,
        /// The ID of the edge to remove
        edge_id: EdgeId,
    },
}

/// Result type for graph operations
pub type GraphCommandResult<T> = Result<T, GraphCommandError>;

/// Errors that can occur during graph command processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommandError {
    /// Graph not found
    GraphNotFound(GraphId),
    /// Node not found
    NodeNotFound(NodeId),
    /// Edge not found
    EdgeNotFound(EdgeId),
    /// Invalid command parameters
    InvalidCommand(String),
    /// Business rule violation
    BusinessRuleViolation(String),
    /// Concurrent modification detected
    ConcurrentModification(String),
}

impl std::fmt::Display for GraphCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphCommandError::GraphNotFound(id) => write!(f, "Graph not found: {}", id),
            GraphCommandError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            GraphCommandError::EdgeNotFound(id) => write!(f, "Edge not found: {}", id),
            GraphCommandError::InvalidCommand(msg) => write!(f, "Invalid command: {}", msg),
            GraphCommandError::BusinessRuleViolation(msg) => write!(f, "Business rule violation: {}", msg),
            GraphCommandError::ConcurrentModification(msg) => write!(f, "Concurrent modification: {}", msg),
        }
    }
}

impl std::error::Error for GraphCommandError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Coverage
    ///
    /// ```mermaid
    /// graph TD
    ///     C[Commands] --> GC[GraphCommand]
    ///     C --> NC[NodeCommand]
    ///     C --> EC[EdgeCommand]
    ///     GC --> S[Serialization]
    ///     NC --> S
    ///     EC --> S
    /// ```

    #[test]
    fn test_graph_command_serialization() {
        let cmd = GraphCommand::CreateGraph {
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: GraphCommand = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            GraphCommand::CreateGraph { name, description, .. } => {
                assert_eq!(name, "Test Graph");
                assert_eq!(description, "A test graph");
            }
            _ => panic!("Expected CreateGraph command"),
        }
    }

    #[test]
    fn test_node_command_serialization() {
        let cmd = NodeCommand::Add {
            graph_id: GraphId::new(),
            node_type: "task".to_string(),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: NodeCommand = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            NodeCommand::Add { node_type, .. } => {
                assert_eq!(node_type, "task");
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_edge_command_serialization() {
        let cmd = EdgeCommand::Add {
            graph_id: GraphId::new(),
            source_id: NodeId::new(),
            target_id: NodeId::new(),
            edge_type: "sequence".to_string(),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: EdgeCommand = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            EdgeCommand::Add { edge_type, .. } => {
                assert_eq!(edge_type, "sequence");
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_graph_command_error_display() {
        let graph_id = GraphId::new();
        let error = GraphCommandError::GraphNotFound(graph_id);
        
        let display = format!("{}", error);
        assert!(display.contains("Graph not found"));
        assert!(display.contains(&graph_id.to_string()));
    }
}
