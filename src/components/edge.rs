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

/// Types of edges
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Sequential flow
    Sequence,
    /// Conditional flow
    Conditional { condition: String },
    /// Parallel flow
    Parallel,
    /// Similarity relationship
    Similarity,
    /// Hierarchical relationship
    Hierarchy,
    /// Association
    Association { relation_type: String },
    /// Dependency
    Dependency,
    /// General purpose edge
    General,
}

impl Default for EdgeType {
    fn default() -> Self {
        Self::General
    }
}

/// Edge relationship details
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRelationship {
    pub edge_type: EdgeType,
    pub label: String,
    pub bidirectional: bool,
}

impl Default for EdgeRelationship {
    fn default() -> Self {
        Self {
            edge_type: EdgeType::default(),
            label: String::new(),
            bidirectional: false,
        }
    }
}

/// Edge metadata
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl Default for EdgeMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Edge weight for algorithms
#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EdgeWeight {
    pub value: f32,
}

impl Default for EdgeWeight {
    fn default() -> Self {
        Self { value: 1.0 }
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