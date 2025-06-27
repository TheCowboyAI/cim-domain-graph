//! Node-level ECS components

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{NodeId, GraphId};

/// Core node entity component
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeEntity {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Types of nodes
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// Workflow step
    WorkflowStep { step_type: String },
    /// Decision point
    Decision { criteria: String },
    /// Integration point
    Integration { system: String },
    /// Concept in knowledge graph
    Concept { category: String },
    /// Event in event flow
    Event { event_type: String },
    /// Feature in development graph
    Feature { status: String },
    /// General purpose node
    General,
}

impl Default for NodeType {
    fn default() -> Self {
        Self::General
    }
}

/// Node content
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeContent {
    pub title: String,
    pub description: String,
    pub data: serde_json::Value,
}

impl Default for NodeContent {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            data: serde_json::Value::Null,
        }
    }
}

/// Node metadata
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl Default for NodeMetadata {
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

/// Node status
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is being created
    Creating,
    /// Node is active
    Active,
    /// Node is selected
    Selected,
    /// Node is highlighted
    Highlighted,
    /// Node is disabled
    Disabled,
    /// Node is hidden
    Hidden,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Node category for grouping
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeCategory {
    pub primary: String,
    pub secondary: Vec<String>,
}

impl Default for NodeCategory {
    fn default() -> Self {
        Self {
            primary: "uncategorized".to_string(),
            secondary: Vec::new(),
        }
    }
} 