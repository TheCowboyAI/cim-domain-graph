//! Edge management systems

use crate::{components::*, events::*, EdgeId};
use bevy_ecs::prelude::*;

/// System that adds edges from EdgeAdded events
pub fn add_edge_system(
    mut commands: Commands,
    mut events: EventReader<EdgeAdded>,
    node_query: Query<&NodeEntity>,
) {
    for event in events.read() {
        // Verify both nodes exist
        let source_exists = node_query
            .iter()
            .any(|n| n.node_id == event.source && n.graph_id == event.graph_id);
        let target_exists = node_query
            .iter()
            .any(|n| n.node_id == event.target && n.graph_id == event.graph_id);

        if !source_exists || !target_exists {
            tracing::warn!(
                "Attempted to add edge between non-existent nodes: {:?} -> {:?}",
                event.source,
                event.target
            );
            continue;
        }

        // Create the edge entity
        let mut entity_builder = commands.spawn((
            EdgeEntity {
                edge_id: event.edge_id,
                source: event.source,
                target: event.target,
                graph_id: event.graph_id,
            },
            event.relationship.clone(),
            EdgeMetadata {
                tags: vec![],
                properties: event.metadata.clone(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
        ));

        // Add visual components
        entity_builder.insert((
            EdgeStyle::Solid,
            EdgeColor::default(),
            EdgeWeight(event.relationship.weight().unwrap_or(1.0)),
        ));

        // Add edge type if specified
        if let Some(edge_type) = parse_edge_type(event.metadata.get("edge_type")) {
            entity_builder.insert(edge_type);
        }

        tracing::debug!(
            "Edge added: {:?} from {:?} to {:?}",
            event.edge_id,
            event.source,
            event.target
        );
    }
}

/// System that updates edges from EdgeUpdated events
pub fn update_edge_system(
    mut events: EventReader<EdgeUpdated>,
    mut edge_query: Query<(
        &EdgeEntity,
        &mut EdgeRelationship,
        &mut EdgeMetadata,
        Option<&mut EdgeWeight>,
    )>,
) {
    for event in events.read() {
        // Find and update the edge
        for (entity, mut relationship, mut metadata, weight) in edge_query.iter_mut() {
            if entity.edge_id == event.edge_id {
                // Update relationship if provided
                if let Some(new_rel) = &event.relationship {
                    *relationship = new_rel.clone();

                    // Update weight if the relationship has one
                    if let Some(mut weight) = weight {
                        weight.0 = new_rel.weight().unwrap_or(weight.0);
                    }
                }

                // Update metadata
                for (key, value) in &event.metadata {
                    metadata.properties.insert(key.clone(), value.clone());
                }
                metadata.updated_at = std::time::SystemTime::now();

                tracing::debug!("Edge updated: {:?}", event.edge_id);
                break;
            }
        }
    }
}

/// System that removes edges from EdgeRemoved events
pub fn remove_edge_system(
    mut commands: Commands,
    mut events: EventReader<EdgeRemoved>,
    edge_query: Query<(Entity, &EdgeEntity)>,
) {
    for event in events.read() {
        // Find and remove the edge
        for (entity, edge) in edge_query.iter() {
            if edge.edge_id == event.edge_id {
                commands.entity(entity).despawn();
                tracing::debug!("Edge removed: {:?}", event.edge_id);
                break;
            }
        }
    }
}

/// System that validates edge connections
pub fn validate_edges_system(
    edge_query: Query<(Entity, &EdgeEntity)>,
    node_query: Query<&NodeEntity>,
    mut commands: Commands,
) {
    // Check all edges have valid connections
    let edges_to_remove: Vec<(Entity, EdgeId)> = edge_query
        .iter()
        .filter_map(|(entity, edge)| {
            let source_exists = node_query
                .iter()
                .any(|n| n.node_id == edge.source && n.graph_id == edge.graph_id);
            let target_exists = node_query
                .iter()
                .any(|n| n.node_id == edge.target && n.graph_id == edge.graph_id);

            if !source_exists || !target_exists {
                Some((entity, edge.edge_id))
            } else {
                None
            }
        })
        .collect();

    // Remove orphaned edges
    for (entity, edge_id) in edges_to_remove {
        tracing::warn!("Removing orphaned edge: {:?}", edge_id);
        commands.entity(entity).despawn();
    }
}

/// Helper function to parse edge type from metadata
fn parse_edge_type(edge_type_value: Option<&serde_json::Value>) -> Option<EdgeType> {
    edge_type_value
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "directed" => Some(EdgeType::Directed),
            "undirected" => Some(EdgeType::Undirected),
            "bidirectional" => Some(EdgeType::Bidirectional),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<EdgeAdded>>();
        world.init_resource::<Events<EdgeUpdated>>();
        world.init_resource::<Events<EdgeRemoved>>();
        world
    }

    fn create_test_nodes(world: &mut World, graph_id: GraphId) -> (NodeId, NodeId) {
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        world.spawn((
            NodeEntity {
                node_id: node1,
                graph_id,
            },
            Position3D::default(),
        ));

        world.spawn((
            NodeEntity {
                node_id: node2,
                graph_id,
            },
            Position3D::default(),
        ));

        (node1, node2)
    }

    #[test]
    fn test_add_edge_system() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();
        let (source, target) = create_test_nodes(&mut world, graph_id);
        let edge_id = EdgeId::new();

        // Send EdgeAdded event
        let mut metadata = HashMap::new();
        metadata.insert("edge_type".to_string(), serde_json::json!("directed"));

        world.resource_mut::<Events<EdgeAdded>>().send(EdgeAdded {
            graph_id,
            edge_id,
            source,
            target,
            relationship: EdgeRelationship::Dependency {
                dependency_type: "requires".to_string(),
                strength: 0.8,
            },
            edge_type: "directed".to_string(),
            metadata,
        });

        // Run the system
        let mut system = IntoSystem::into_system(add_edge_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify edge was created
        let mut query = world.query::<(&EdgeEntity, &EdgeRelationship, &EdgeType, &EdgeWeight)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (entity, relationship, edge_type, weight) = results[0];
        assert_eq!(entity.edge_id, edge_id);
        assert_eq!(entity.source, source);
        assert_eq!(entity.target, target);
        assert!(matches!(relationship, EdgeRelationship::Dependency { .. }));
        assert_eq!(*edge_type, EdgeType::Directed);
        assert_eq!(weight.0, 0.8);
    }

    #[test]
    fn test_update_edge_system() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();
        let edge_id = EdgeId::new();

        // Create an edge
        world.spawn((
            EdgeEntity {
                edge_id,
                source: NodeId::new(),
                target: NodeId::new(),
                graph_id,
            },
            EdgeRelationship::Association {
                association_type: "related".to_string(),
            },
            EdgeMetadata {
                tags: vec![],
                properties: HashMap::new(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            EdgeWeight(1.0),
        ));

        // Send update event
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), serde_json::json!("high"));

        world
            .resource_mut::<Events<EdgeUpdated>>()
            .send(EdgeUpdated {
                graph_id,
                edge_id,
                relationship: Some(EdgeRelationship::Dependency {
                    dependency_type: "requires".to_string(),
                    strength: 0.9,
                }),
                metadata,
            });

        // Run the system
        let mut system = IntoSystem::into_system(update_edge_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify updates
        let mut query = world.query::<(&EdgeRelationship, &EdgeMetadata, &EdgeWeight)>();
        let (relationship, metadata, weight) = query.single(&world).unwrap();

        assert!(matches!(relationship, EdgeRelationship::Dependency { .. }));
        assert_eq!(weight.0, 0.9);
        assert_eq!(
            metadata.properties.get("priority"),
            Some(&serde_json::json!("high"))
        );
    }

    #[test]
    fn test_remove_edge_system() {
        let mut world = setup_test_world();
        let edge_id = EdgeId::new();
        let graph_id = GraphId::new();

        // Create an edge
        let edge_entity = world
            .spawn((EdgeEntity {
                edge_id,
                source: NodeId::new(),
                target: NodeId::new(),
                graph_id,
            },))
            .id();

        // Send remove event
        world
            .resource_mut::<Events<EdgeRemoved>>()
            .send(EdgeRemoved { graph_id, edge_id });

        // Run the system
        let mut system = IntoSystem::into_system(remove_edge_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify edge was removed
        assert!(world.get_entity(edge_entity).is_err());
    }

    #[test]
    fn test_edge_validation_prevents_invalid_edges() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();
        let node_id = NodeId::new();

        // Create only one node
        world.spawn((NodeEntity { node_id, graph_id }, Position3D::default()));

        // Try to add edge to non-existent node
        world.resource_mut::<Events<EdgeAdded>>().send(EdgeAdded {
            graph_id,
            edge_id: EdgeId::new(),
            source: node_id,
            target: NodeId::new(), // Non-existent
            relationship: EdgeRelationship::Association {
                association_type: "test".to_string(),
            },
            edge_type: "directed".to_string(),
            metadata: HashMap::new(),
        });

        // Run the system
        let mut system = IntoSystem::into_system(add_edge_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify no edge was created
        let mut edge_query = world.query::<&EdgeEntity>();
        assert_eq!(edge_query.iter(&world).count(), 0);
    }
}
