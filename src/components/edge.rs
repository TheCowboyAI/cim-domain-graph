//! Edge-level ECS components

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{EdgeId, NodeId, GraphId};

/// Core edge entity component
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeEntity {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub source: NodeId,
    pub target: NodeId,
}

/// Edge type
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    /// Directed edge (one-way)
    Directed,
    /// Undirected edge (two-way)
    Undirected,
    /// Bidirectional edge (explicitly two-way)
    Bidirectional,
    /// Workflow edge
    Workflow {
        condition: Option<String>,
    },
    /// Data flow edge
    DataFlow {
        data_type: String,
    },
    /// Control flow edge
    ControlFlow,
}

/// Edge relationship types
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeRelationship {
    /// Dependency relationship
    Dependency {
        dependency_type: String,
        strength: f32,
    },
    /// Similarity relationship
    Similarity {
        score: f32,
    },
    /// Hierarchical relationship
    Hierarchy {
        parent_to_child: bool,
    },
    /// Association relationship
    Association {
        association_type: String,
    },
    /// Flow relationship
    Flow {
        flow_type: String,
        capacity: Option<f32>,
    },
}

impl EdgeRelationship {
    /// Get the weight associated with this relationship
    pub fn weight(&self) -> Option<f32> {
        match self {
            Self::Dependency { strength, .. } => Some(*strength),
            Self::Similarity { score } => Some(*score),
            Self::Hierarchy { .. } => Some(1.0),
            Self::Association { .. } => Some(1.0),
            Self::Flow { .. } => Some(1.0),
        }
    }
}

/// Edge relationship details (deprecated - use EdgeRelationship directly)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRelationshipOld {
    pub edge_type: EdgeType,
    pub label: String,
    pub bidirectional: bool,
}

impl Default for EdgeRelationshipOld {
    fn default() -> Self {
        Self {
            edge_type: EdgeType::Directed,
            label: String::new(),
            bidirectional: false,
        }
    }
}

/// Edge metadata
#[derive(Component, Debug, Clone)]
pub struct EdgeMetadata {
    pub tags: Vec<String>,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl Default for EdgeMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            tags: Vec::new(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Edge weight
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EdgeWeight(pub f32);

impl Default for EdgeWeight {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Edge direction for visualization
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeDirection {
    /// From source to target
    Forward,
    /// From target to source
    Backward,
    /// Both directions
    Bidirectional,
    /// No direction
    Undirected,
}

impl Default for EdgeDirection {
    fn default() -> Self {
        Self::Forward
    }
}

/// Edge style for rendering
#[derive(Component, Debug, Clone, PartialEq)]
pub enum EdgeStyle {
    Solid,
    Dashed,
    Dotted,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self::Solid
    }
}

/// Edge color
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EdgeColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for EdgeColor {
    fn default() -> Self {
        Self {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }
    }
} 