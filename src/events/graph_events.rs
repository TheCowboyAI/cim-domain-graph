//! Graph domain events

use bevy_ecs::prelude::*;
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::value_objects::Position3D;

// Re-export identifiers that will be moved here eventually
pub use cim_domain::{NodeId, EdgeId};
pub use cim_domain::GraphId;

/// Graph created event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct GraphCreated {
    /// The unique identifier of the graph
    pub graph_id: GraphId,
    /// The name of the graph
    pub name: String,
    /// A description of the graph's purpose
    pub description: String,
    /// Graph type
    pub graph_type: Option<crate::components::GraphType>,
    /// Additional metadata about the graph
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the graph was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Graph updated event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct GraphUpdated {
    pub graph_id: GraphId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Graph archived event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct GraphArchived {
    pub graph_id: GraphId,
    pub archived_at: chrono::DateTime<chrono::Utc>,
}

/// Node added event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct NodeAdded {
    /// The graph to which the node was added
    pub graph_id: GraphId,
    /// The unique identifier of the node
    pub node_id: NodeId,
    /// The position of the node
    pub position: Position3D,
    /// The type of node (e.g., "task", "decision", "gateway")
    pub node_type: String,
    /// Additional metadata about the node
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node updated event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct NodeUpdated {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub position: Option<Position3D>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node removed event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct NodeRemoved {
    /// The graph from which the node was removed
    pub graph_id: GraphId,
    /// The ID of the node that was removed
    pub node_id: NodeId,
}



/// Edge added event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeAdded {
    /// The graph to which the edge was added
    pub graph_id: GraphId,
    /// The unique identifier of the edge
    pub edge_id: EdgeId,
    /// The source node of the edge
    pub source: NodeId,
    /// The target node of the edge
    pub target: NodeId,
    /// The type of edge (e.g., "sequence", "conditional", "parallel")
    pub edge_type: String,
    /// Additional metadata about the edge
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge updated event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeUpdated {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge removed event
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
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
        "graphs.graph.created.v1".to_string()
    }
}

impl DomainEvent for GraphUpdated {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "GraphUpdated"
    }

    fn subject(&self) -> String {
        "graphs.graph.updated.v1".to_string()
    }
}

impl DomainEvent for GraphArchived {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "GraphArchived"
    }

    fn subject(&self) -> String {
        "graphs.graph.archived.v1".to_string()
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
        "graphs.node.added.v1".to_string()
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
        "graphs.node.updated.v1".to_string()
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
        "graphs.node.removed.v1".to_string()
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
        "graphs.edge.added.v1".to_string()
    }
}

impl DomainEvent for EdgeUpdated {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "EdgeUpdated"
    }

    fn subject(&self) -> String {
        "graphs.edge.updated.v1".to_string()
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
        "graphs.edge.removed.v1".to_string()
    }
}
