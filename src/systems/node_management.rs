//! Node management systems

use bevy_ecs::prelude::*;
use crate::{
    components::*,
    events::*,
    value_objects::Position3D,
};

/// System that adds new nodes from NodeAdded events
pub fn add_node_system(
    mut commands: Commands,
    mut events: EventReader<NodeAdded>,
    graph_query: Query<&GraphEntity>,
) {
    for event in events.read() {
        // Verify the graph exists
        let graph_exists = graph_query.iter()
            .any(|g| g.graph_id == event.graph_id);
        
        if !graph_exists {
            tracing::warn!("Attempted to add node to non-existent graph: {:?}", event.graph_id);
            continue;
        }
        
        // Create the node entity with core components
        let mut entity_builder = commands.spawn((
            NodeEntity {
                node_id: event.node_id,
                graph_id: event.graph_id,
            },
            event.position.clone(),
            NodeStatus::Active,
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
        ));
        
        // Add visual components
        entity_builder.insert((
            Color::rgb(0.3, 0.5, 0.8),
            Size::new(1.0, 1.0, 1.0),
            Visibility::Visible,
        ));
        
        // Add content if provided
        if let Some(title) = event.metadata.get("title").and_then(|v| v.as_str()) {
            entity_builder.insert(NodeContent {
                title: title.to_string(),
                description: event.metadata.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data: event.metadata.get("data")
                    .cloned()
                    .unwrap_or(serde_json::json!({})),
            });
        }
        
        // Add node type if specified
        if let Some(node_type) = parse_node_type(&event.node_type) {
            entity_builder.insert(node_type);
        }
        
        tracing::debug!("Node added: {:?} at position {:?}", event.node_id, event.position);
    }
}

/// System that updates nodes from NodeUpdated events
pub fn update_node_system(
    mut events: EventReader<NodeUpdated>,
    mut node_query: Query<(
        &NodeEntity,
        &mut Position3D,
        &mut NodeMetadata,
        Option<&mut NodeContent>,
        Option<&mut NodeStatus>,
    )>,
) {
    for event in events.read() {
        // Find and update the node
        for (entity, mut position, mut metadata, content, status) in node_query.iter_mut() {
            if entity.node_id == event.node_id {
                // Update position if provided
                if let Some(new_pos) = &event.position {
                    *position = new_pos.clone();
                }
                
                // Update metadata
                for (key, value) in &event.metadata {
                    metadata.properties.insert(key.clone(), value.clone());
                }
                metadata.updated_at = std::time::SystemTime::now();
                
                // Update content if present
                if let Some(mut content) = content {
                    if let Some(title) = event.metadata.get("title").and_then(|v| v.as_str()) {
                        content.title = title.to_string();
                    }
                    if let Some(desc) = event.metadata.get("description").and_then(|v| v.as_str()) {
                        content.description = desc.to_string();
                    }
                    if let Some(data) = event.metadata.get("data") {
                        content.data = data.clone();
                    }
                }
                
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
                
                tracing::debug!("Node updated: {:?}", event.node_id);
                break;
            }
        }
    }
}

/// System that removes nodes from NodeRemoved events
pub fn remove_node_system(
    mut commands: Commands,
    mut events: EventReader<NodeRemoved>,
    node_query: Query<(Entity, &NodeEntity)>,
    edge_query: Query<(Entity, &EdgeEntity)>,
) {
    for event in events.read() {
        // Find and remove the node entity
        for (entity, node) in node_query.iter() {
            if node.node_id == event.node_id {
                commands.entity(entity).despawn();
                
                // Also remove any edges connected to this node
                for (edge_entity, edge) in edge_query.iter() {
                    if edge.source == event.node_id || edge.target == event.node_id {
                        commands.entity(edge_entity).despawn();
                        tracing::debug!("Removed edge connected to node: {:?}", event.node_id);
                    }
                }
                
                tracing::debug!("Node removed: {:?}", event.node_id);
                break;
            }
        }
    }
}

/// Helper function to parse node type from string
fn parse_node_type(type_str: &str) -> Option<NodeType> {
    match type_str {
        "start" => Some(NodeType::Start),
        "end" => Some(NodeType::End),
        "process" => Some(NodeType::Process),
        "decision" => Some(NodeType::Decision),
        "data" => Some(NodeType::Data),
        "event" => Some(NodeType::Event),
        "gateway" => Some(NodeType::Gateway),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<NodeAdded>>();
        world.init_resource::<Events<NodeUpdated>>();
        world.init_resource::<Events<NodeRemoved>>();
        world
    }

    #[test]
    fn test_add_node_system() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();
        let node_id = NodeId::new();
        
        // Create a graph first
        world.spawn((
            GraphEntity {
                graph_id,
                graph_type: GraphType::General,
            },
        ));
        
        // Send NodeAdded event
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), serde_json::json!("Test Node"));
        metadata.insert("description".to_string(), serde_json::json!("A test node"));
        
        world.resource_mut::<Events<NodeAdded>>().send(NodeAdded {
            graph_id,
            node_id,
            position: Position3D::new(10.0, 20.0, 30.0),
            node_type: "process".to_string(),
            metadata,
        });
        
        // Run the system
        let mut system = IntoSystem::into_system(add_node_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);
        
        // Verify node was created
        let mut query = world.query::<(&NodeEntity, &Position3D, &NodeType, &NodeContent)>();
        let results: Vec<_> = query.iter(&world).collect();
        
        assert_eq!(results.len(), 1);
        let (entity, position, node_type, content) = results[0];
        assert_eq!(entity.node_id, node_id);
        assert_eq!(entity.graph_id, graph_id);
        assert_eq!(*position, Position3D::new(10.0, 20.0, 30.0));
        assert_eq!(*node_type, NodeType::Process);
        assert_eq!(content.title, "Test Node");
        assert_eq!(content.description, "A test node");
    }

    #[test]
    fn test_update_node_system() {
        let mut world = setup_test_world();
        let node_id = NodeId::new();
        let graph_id = GraphId::new();
        
        // Create a node
        world.spawn((
            NodeEntity { node_id, graph_id },
            Position3D::new(0.0, 0.0, 0.0),
            NodeMetadata {
                tags: vec![],
                properties: HashMap::new(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            NodeContent {
                title: "Original".to_string(),
                description: String::new(),
                data: serde_json::json!({}),
            },
            NodeStatus::Active,
        ));
        
        // Send update event
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), serde_json::json!("Updated"));
        metadata.insert("status".to_string(), serde_json::json!("selected"));
        metadata.insert("custom".to_string(), serde_json::json!("value"));
        
        world.resource_mut::<Events<NodeUpdated>>().send(NodeUpdated {
            graph_id,
            node_id,
            position: Some(Position3D::new(50.0, 50.0, 0.0)),
            metadata,
        });
        
        // Run the system
        let mut system = IntoSystem::into_system(update_node_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);
        
        // Verify updates
        let mut query = world.query::<(&Position3D, &NodeMetadata, &NodeContent, &NodeStatus)>();
        let (position, metadata, content, status) = query.single(&world).unwrap();
        
        assert_eq!(*position, Position3D::new(50.0, 50.0, 0.0));
        assert_eq!(content.title, "Updated");
        assert_eq!(*status, NodeStatus::Selected);
        assert_eq!(metadata.properties.get("custom"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_remove_node_system_with_edges() {
        let mut world = setup_test_world();
        let node_id = NodeId::new();
        let node_id2 = NodeId::new();
        let graph_id = GraphId::new();
        
        // Create nodes
        world.spawn((
            NodeEntity { node_id, graph_id },
            Position3D::default(),
        ));
        
        world.spawn((
            NodeEntity { node_id: node_id2, graph_id },
            Position3D::default(),
        ));
        
        // Create edges connected to the first node
        let edge1 = world.spawn((
            EdgeEntity {
                edge_id: crate::EdgeId::new(),
                source: node_id,
                target: node_id2,
                graph_id,
            },
        )).id();
        
        let edge2 = world.spawn((
            EdgeEntity {
                edge_id: crate::EdgeId::new(),
                source: node_id2,
                target: node_id,
                graph_id,
            },
        )).id();
        
        // Send remove event
        world.resource_mut::<Events<NodeRemoved>>().send(NodeRemoved {
            graph_id,
            node_id,
        });
        
        // Run the system
        let mut system = IntoSystem::into_system(remove_node_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);
        
        // Verify node was removed
        let mut node_query = world.query::<&NodeEntity>();
        let remaining_nodes: Vec<_> = node_query.iter(&world).collect();
        assert_eq!(remaining_nodes.len(), 1);
        assert_eq!(remaining_nodes[0].node_id, node_id2);
        
        // Verify edges connected to removed node were also removed
        assert!(world.get_entity(edge1).is_err());
        assert!(world.get_entity(edge2).is_err());
    }
} 