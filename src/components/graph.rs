//! Graph-level ECS components

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::GraphId;

/// Core graph entity component
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphEntity {
    pub graph_id: GraphId,
    pub graph_type: GraphType,
}

/// Graph types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphType {
    /// General purpose graph
    Generic,
    /// Workflow graph
    Workflow,
    /// Knowledge graph
    Knowledge,
    /// Development graph
    Development,
    /// Event flow graph
    EventFlow,
    /// General graph (alias for Generic)
    General,
}

impl Default for GraphType {
    fn default() -> Self {
        Self::Generic
    }
}

/// Graph status
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphStatus {
    /// Graph is being created
    Creating,
    /// Graph is active and can be modified
    Active,
    /// Graph is read-only
    ReadOnly,
    /// Graph is archived
    Archived,
}

impl Default for GraphStatus {
    fn default() -> Self {
        Self::Creating
    }
}

/// Graph metadata
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            name: String::new(),
            description: String::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayoutDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

/// Graph layout algorithms
#[derive(Component, Debug, Clone, PartialEq)]
pub enum GraphLayout {
    /// Force-directed layout
    ForceDirected {
        spring_strength: f32,
        repulsion_strength: f32,
        damping: f32,
    },
    /// Hierarchical layout
    Hierarchical {
        direction: LayoutDirection,
        layer_spacing: f32,
        node_spacing: f32,
    },
    /// Circular layout
    Circular {
        radius: f32,
    },
    /// Grid layout
    Grid {
        columns: usize,
        spacing: f32,
    },
    /// Random layout
    Random {
        bounds: (f32, f32, f32),
    },
}

impl Default for GraphLayout {
    fn default() -> Self {
        Self::ForceDirected {
            spring_strength: 0.1,
            repulsion_strength: 100.0,
            damping: 0.9,
        }
    }
} 