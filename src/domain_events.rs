//! Domain events enum for graph domain

use crate::events::{GraphCreated, NodeAdded, NodeRemoved, EdgeAdded, EdgeRemoved};
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};

/// Enum wrapper for graph domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphDomainEvent {
    /// A new graph was created
    GraphCreated(GraphCreated),
    /// A node was added to a graph
    NodeAdded(NodeAdded),
    /// A node was removed from a graph
    NodeRemoved(NodeRemoved),
    /// An edge was added between nodes
    EdgeAdded(EdgeAdded),
    /// An edge was removed from the graph
    EdgeRemoved(EdgeRemoved),
}

impl DomainEvent for GraphDomainEvent {
    fn subject(&self) -> String {
        match self {
            Self::GraphCreated(e) => e.subject(),
            Self::NodeAdded(e) => e.subject(),
            Self::NodeRemoved(e) => e.subject(),
            Self::EdgeAdded(e) => e.subject(),
            Self::EdgeRemoved(e) => e.subject(),
        }
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            Self::GraphCreated(e) => e.aggregate_id(),
            Self::NodeAdded(e) => e.aggregate_id(),
            Self::NodeRemoved(e) => e.aggregate_id(),
            Self::EdgeAdded(e) => e.aggregate_id(),
            Self::EdgeRemoved(e) => e.aggregate_id(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::GraphCreated(e) => e.event_type(),
            Self::NodeAdded(e) => e.event_type(),
            Self::NodeRemoved(e) => e.event_type(),
            Self::EdgeAdded(e) => e.event_type(),
            Self::EdgeRemoved(e) => e.event_type(),
        }
    }
}
