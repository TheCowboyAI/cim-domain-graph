//! Node management systems

use bevy_ecs::prelude::*;
use crate::{
    components::{
        NodeEntity, NodeType, NodeContent, NodeMetadata, NodeStatus, NodeCategory,
        Position3D, Color, Size, Style, Visibility,
    },
    events::{NodeAdded, NodeUpdated, NodeRemoved},
};

/// System that adds new nodes from NodeAdded events
pub fn add_node_system(
    mut commands: Commands,
    mut events: EventReader<NodeAdded>,
) {
    for event in events.read() {
        // Create the node entity with core components
        let mut entity_builder = commands.spawn((
            NodeEntity {
                node_id: event.node_id,
                graph_id: event.graph_id,
            },
            NodeContent {
                title: event.metadata.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string(),
                description: event.metadata.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data: event.metadata.get("data")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            },
            NodeMetadata {
                tags: event.metadata.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect())
                    .unwrap_or_default(),
                properties: event.metadata.clone(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            NodeStatus::Active,
            NodeCategory::default(),
        ));
        
        // Add visual components separately
        entity_builder.insert((
            event.position,
            Color::default(),
            Size::default(),
            Style::default(),
            Visibility::default(),
        ));
        
        // Add node type if specified
        if let Some(node_type_str) = event.metadata.get("node_type").and_then(|v| v.as_str()) {
            match node_type_str {
                "workflow_step" => {
                    entity_builder.insert(NodeType::WorkflowStep {
                        step_type: event.metadata.get("step_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default")
                            .to_string(),
                    });
                }
                "decision" => {
                    entity_builder.insert(NodeType::Decision {
                        criteria: event.metadata.get("criteria")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    });
                }
                "concept" => {
                    entity_builder.insert(NodeType::Concept {
                        category: event.metadata.get("category")
                            .and_then(|v| v.as_str())
                            .unwrap_or("general")
                            .to_string(),
                    });
                }
                _ => {}
            }
        }
    }
}

/// System that updates nodes from NodeUpdated events
pub fn update_node_system(
    mut events: EventReader<NodeUpdated>,
    mut node_query: Query<(
        &NodeEntity,
        &mut NodeContent,
        &mut NodeMetadata,
        &mut Position3D,
        Option<&mut NodeStatus>,
    )>,
) {
    for event in events.read() {
        // Find and update the node
        for (entity, mut content, mut metadata, mut position, status) in node_query.iter_mut() {
            if entity.node_id == event.node_id {
                // Update position if provided
                if let Some(new_pos) = event.position {
                    *position = new_pos;
                }
                
                // Update content from metadata
                if let Some(title) = event.metadata.get("title").and_then(|v| v.as_str()) {
                    content.title = title.to_string();
                }
                if let Some(desc) = event.metadata.get("description").and_then(|v| v.as_str()) {
                    content.description = desc.to_string();
                }
                if let Some(data) = event.metadata.get("data") {
                    content.data = data.clone();
                }
                
                // Update metadata
                for (key, value) in &event.metadata {
                    metadata.properties.insert(key.clone(), value.clone());
                }
                metadata.updated_at = std::time::SystemTime::now();
                
                // Update status if provided
                if let Some(mut status) = status {
                    if let Some(status_str) = event.metadata.get("status").and_then(|v| v.as_str()) {
                        *status = match status_str {
                            "active" => NodeStatus::Active,
                            "selected" => NodeStatus::Selected,
                            "highlighted" => NodeStatus::Highlighted,
                            "disabled" => NodeStatus::Disabled,
                            "hidden" => NodeStatus::Hidden,
                            _ => NodeStatus::Active,
                        };
                    }
                }
            }
        }
    }
}

/// System that removes nodes from NodeRemoved events
pub fn remove_node_system(
    mut commands: Commands,
    mut events: EventReader<NodeRemoved>,
    node_query: Query<(Entity, &NodeEntity)>,
) {
    for event in events.read() {
        // Find and remove the node entity
        for (entity, node) in node_query.iter() {
            if node.node_id == event.node_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System that processes node events for logging/debugging
pub fn process_node_events_system(
    mut node_added: EventReader<NodeAdded>,
    mut node_updated: EventReader<NodeUpdated>,
    mut node_removed: EventReader<NodeRemoved>,
) {
    for event in node_added.read() {
        tracing::debug!("Node added: {:?} at position {:?}", event.node_id, event.position);
    }
    
    for event in node_updated.read() {
        tracing::debug!("Node updated: {:?}", event.node_id);
    }
    
    for event in node_removed.read() {
        tracing::debug!("Node removed: {:?}", event.node_id);
    }
} 