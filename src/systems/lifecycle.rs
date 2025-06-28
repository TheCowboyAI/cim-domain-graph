//! Graph lifecycle systems

use crate::{components::*, events::*};
use bevy_ecs::prelude::*;

/// System that processes GraphCreated events and spawns graph entities
pub fn create_graph_system(mut commands: Commands, mut events: EventReader<GraphCreated>) {
    for event in events.read() {
        // Spawn the graph entity with all required components
        commands.spawn((
            GraphEntity {
                graph_id: event.graph_id,
                graph_type: event.graph_type.unwrap_or(GraphType::Generic),
            },
            GraphStatus::Active,
            GraphMetadata {
                name: event.name.clone(),
                description: event.description.clone(),
                tags: vec![],
                properties: event.metadata.clone(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            GraphLayout::ForceDirected {
                spring_strength: 0.1,
                repulsion_strength: 100.0,
                damping: 0.9,
            },
        ));
    }
}

/// System that processes GraphUpdated events and updates graph components
pub fn update_graph_system(
    mut events: EventReader<GraphUpdated>,
    mut query: Query<(&GraphEntity, &mut GraphMetadata)>,
) {
    for event in events.read() {
        // Find the graph entity and update its metadata
        for (entity, mut metadata) in query.iter_mut() {
            if entity.graph_id == event.graph_id {
                if let Some(name) = &event.name {
                    metadata.name = name.clone();
                }
                if let Some(desc) = &event.description {
                    metadata.description = desc.clone();
                }
                // Merge metadata
                for (key, value) in &event.metadata {
                    metadata.properties.insert(key.clone(), value.clone());
                }
                metadata.updated_at = std::time::SystemTime::now();
            }
        }
    }
}

/// System that processes GraphArchived events and updates graph status
pub fn archive_graph_system(
    mut events: EventReader<GraphArchived>,
    mut query: Query<(&GraphEntity, &mut GraphStatus)>,
) {
    for event in events.read() {
        // Find the graph entity and archive it
        for (entity, mut status) in query.iter_mut() {
            if entity.graph_id == event.graph_id {
                *status = GraphStatus::Archived;
            }
        }
    }
}

/// System that processes domain events and updates the aggregate
pub fn process_graph_events_system(
    mut graph_created: EventReader<GraphCreated>,
    mut graph_updated: EventReader<GraphUpdated>,
    mut graph_archived: EventReader<GraphArchived>,
) {
    // Process created events
    for event in graph_created.read() {
        // Log or handle aggregate updates if needed
        tracing::info!("Graph created: {:?}", event.graph_id);
    }

    // Process updated events
    for event in graph_updated.read() {
        tracing::info!("Graph updated: {:?}", event.graph_id);
    }

    // Process archived events
    for event in graph_archived.read() {
        tracing::info!("Graph archived: {:?}", event.graph_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<GraphCreated>>();
        world.init_resource::<Events<GraphUpdated>>();
        world.init_resource::<Events<GraphArchived>>();
        world
    }

    #[test]
    fn test_create_graph_system() {
        let mut world = setup_test_world();

        // Send a GraphCreated event
        let graph_id = GraphId::new();
        world
            .resource_mut::<Events<GraphCreated>>()
            .send(GraphCreated {
                graph_id,
                name: "Test Graph".to_string(),
                description: "A test graph".to_string(),
                graph_type: Some(GraphType::Workflow),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
            });

        // Run the system
        let mut system = IntoSystem::into_system(create_graph_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify graph entity was created
        let mut query = world.query::<(&GraphEntity, &GraphStatus, &GraphMetadata)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (entity, status, metadata) = results[0];
        assert_eq!(entity.graph_id, graph_id);
        assert_eq!(entity.graph_type, GraphType::Workflow);
        assert_eq!(*status, GraphStatus::Active);
        assert_eq!(metadata.name, "Test Graph");
    }

    #[test]
    fn test_update_graph_system() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();

        // Create a graph entity
        world.spawn((
            GraphEntity {
                graph_id,
                graph_type: GraphType::Generic,
            },
            GraphStatus::Active,
            GraphMetadata {
                name: "Original".to_string(),
                description: "Original desc".to_string(),
                tags: vec![],
                properties: HashMap::new(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
        ));

        // Send an update event
        let mut new_metadata = HashMap::new();
        new_metadata.insert("key".to_string(), serde_json::json!("value"));

        world
            .resource_mut::<Events<GraphUpdated>>()
            .send(GraphUpdated {
                graph_id,
                name: Some("Updated".to_string()),
                description: None,
                metadata: new_metadata,
            });

        // Run the system
        let mut system = IntoSystem::into_system(update_graph_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify updates
        let mut query = world.query::<&GraphMetadata>();
        let metadata = query.single(&world).unwrap();
        assert_eq!(metadata.name, "Updated");
        assert_eq!(metadata.description, "Original desc"); // Not updated
        assert_eq!(
            metadata.properties.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_archive_graph_system() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();

        // Create an active graph
        world.spawn((
            GraphEntity {
                graph_id,
                graph_type: GraphType::General,
            },
            GraphStatus::Active,
        ));

        // Send archive event
        world
            .resource_mut::<Events<GraphArchived>>()
            .send(GraphArchived {
                graph_id,
                archived_at: chrono::Utc::now(),
            });

        // Run the system
        let mut system = IntoSystem::into_system(archive_graph_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify status changed
        let mut query = world.query::<&GraphStatus>();
        let status = query.single(&world).unwrap();
        assert_eq!(*status, GraphStatus::Archived);
    }
}
