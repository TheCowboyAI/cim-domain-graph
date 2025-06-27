//! Graph lifecycle systems

use bevy_ecs::prelude::*;
use crate::{
    components::{GraphEntity, GraphType, GraphStatus, GraphMetadata, GraphLayout},
    events::{GraphCreated, GraphUpdated, GraphArchived},
};

/// System that creates new graph entities from GraphCreated events
pub fn create_graph_system(
    mut commands: Commands,
    mut events: EventReader<GraphCreated>,
) {
    for event in events.read() {
        // Create the graph entity with all components
        commands.spawn((
            GraphEntity {
                graph_id: event.graph_id,
                graph_type: event.graph_type.unwrap_or(GraphType::General),
            },
            GraphStatus::Active,
            GraphMetadata {
                name: event.name.clone(),
                description: event.description.clone(),
                tags: Vec::new(),
                properties: event.metadata.clone(),
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            GraphLayout::default(),
        ));
    }
}

/// System that updates graph entities from GraphUpdated events
pub fn update_graph_system(
    mut events: EventReader<GraphUpdated>,
    mut graph_query: Query<(&GraphEntity, &mut GraphMetadata, &mut GraphStatus)>,
) {
    for event in events.read() {
        // Find the graph entity
        for (entity, mut metadata, mut status) in graph_query.iter_mut() {
            if entity.graph_id == event.graph_id {
                // Update metadata if provided
                if let Some(name) = &event.name {
                    metadata.name = name.clone();
                }
                if let Some(desc) = &event.description {
                    metadata.description = desc.clone();
                }
                
                // Update properties
                for (key, value) in &event.metadata {
                    metadata.properties.insert(key.clone(), value.clone());
                }
                
                metadata.updated_at = std::time::SystemTime::now();
                
                // Update status if needed
                if event.metadata.get("archived").and_then(|v| v.as_bool()).unwrap_or(false) {
                    *status = GraphStatus::Archived;
                }
            }
        }
    }
}

/// System that archives graph entities
pub fn archive_graph_system(
    mut events: EventReader<GraphArchived>,
    mut graph_query: Query<(&GraphEntity, &mut GraphStatus)>,
) {
    for event in events.read() {
        // Find and archive the graph
        for (entity, mut status) in graph_query.iter_mut() {
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