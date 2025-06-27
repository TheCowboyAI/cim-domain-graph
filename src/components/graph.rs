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

/// Types of graphs in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphType {
    /// Business workflow visualization
    Workflow,
    /// Conceptual knowledge graph
    Conceptual,
    /// Event flow visualization
    EventFlow,
    /// Development progress (dog-fooding)
    Development,
    /// General purpose graph
    General,
}

impl Default for GraphType {
    fn default() -> Self {
        Self::General
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

/// Graph layout configuration
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphLayout {
    pub layout_type: LayoutType,
    pub parameters: LayoutParameters,
}

/// Types of graph layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayoutType {
    /// Force-directed layout
    ForceDirected,
    /// Hierarchical layout
    Hierarchical,
    /// Circular layout
    Circular,
    /// Grid layout
    Grid,
    /// Custom layout
    Custom,
}

impl Default for LayoutType {
    fn default() -> Self {
        Self::ForceDirected
    }
}

/// Layout parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutParameters {
    pub node_spacing: f32,
    pub edge_length: f32,
    pub repulsion_force: f32,
    pub attraction_force: f32,
    pub damping: f32,
    pub iterations: u32,
}

impl Default for LayoutParameters {
    fn default() -> Self {
        Self {
            node_spacing: 100.0,
            edge_length: 150.0,
            repulsion_force: 1000.0,
            attraction_force: 0.1,
            damping: 0.9,
            iterations: 100,
        }
    }
}

impl Default for GraphLayout {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::default(),
            parameters: LayoutParameters::default(),
        }
    }
} 