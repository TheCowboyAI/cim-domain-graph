//! Graph domain events

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// Re-export identifiers that will be moved here eventually
pub use cim_domain::{NodeId, EdgeId};
pub use cim_domain::GraphId;

/// Graph created event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphCreated {
    /// The unique identifier of the graph
    pub graph_id: GraphId,
    /// The name of the graph
    pub name: String,
    /// A description of the graph's purpose
    pub description: String,
    /// Additional metadata about the graph
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the graph was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Node added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAdded {
    /// The graph to which the node was added
    pub graph_id: GraphId,
    /// The unique identifier of the node
    pub node_id: NodeId,
    /// The type of node (e.g., "task", "decision", "gateway")
    pub node_type: String,
    /// Additional metadata about the node
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node removed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRemoved {
    /// The graph from which the node was removed
    pub graph_id: GraphId,
    /// The ID of the node that was removed
    pub node_id: NodeId,
}

/// Node updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUpdated {
    /// The graph containing the updated node
    pub graph_id: GraphId,
    /// The ID of the node that was updated
    pub node_id: NodeId,
    /// The updated metadata for the node
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeAdded {
    /// The graph to which the edge was added
    pub graph_id: GraphId,
    /// The unique identifier of the edge
    pub edge_id: EdgeId,
    /// The source node of the edge
    pub source_id: NodeId,
    /// The target node of the edge
    pub target_id: NodeId,
    /// The type of edge (e.g., "sequence", "conditional", "parallel")
    pub edge_type: String,
    /// Additional metadata about the edge
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge removed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRemoved {
    /// The graph from which the edge was removed
    pub graph_id: GraphId,
    /// The ID of the edge that was removed
    pub edge_id: EdgeId,
}

// Implement DomainEvent trait for all events
impl DomainEvent for GraphCreated {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "GraphCreated"
    }

    fn subject(&self) -> String {
        format!("graphs.graph.created.v1")
    }
}

impl DomainEvent for NodeAdded {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "NodeAdded"
    }

    fn subject(&self) -> String {
        format!("graphs.node.added.v1")
    }
}

impl DomainEvent for NodeRemoved {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "NodeRemoved"
    }

    fn subject(&self) -> String {
        format!("graphs.node.removed.v1")
    }
}

impl DomainEvent for NodeUpdated {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "NodeUpdated"
    }

    fn subject(&self) -> String {
        format!("graphs.node.updated.v1")
    }
}

impl DomainEvent for EdgeAdded {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "EdgeAdded"
    }

    fn subject(&self) -> String {
        format!("graphs.edge.added.v1")
    }
}

impl DomainEvent for EdgeRemoved {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "EdgeRemoved"
    }

    fn subject(&self) -> String {
        format!("graphs.edge.removed.v1")
    }
}
