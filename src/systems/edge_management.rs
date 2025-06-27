//! Edge management systems

use bevy_ecs::prelude::*;
use crate::{
    components::{
        EdgeEntity, EdgeType, EdgeRelationship, EdgeMetadata, EdgeWeight, EdgeDirection,
        NodeEntity,
    },
    events::{EdgeAdded, EdgeUpdated, EdgeRemoved},
};

/// System that connects nodes from EdgeAdded events
pub fn connect_nodes_system(
    mut commands: Commands,
    mut events: EventReader<EdgeAdded>,
    node_query: Query<(Entity, &NodeEntity)>,
) {
    for event in events.read() {
        // Find source and target node entities
        let mut source_entity = None;
        let mut target_entity = None;
        
        for (entity, node) in node_query.iter() {
            if node.node_id == event.source {
                source_entity = Some(entity);
            }
            if node.node_id == event.target {
                target_entity = Some(entity);
            }
        }
        
        // Only create edge if both nodes exist
        if source_entity.is_some() && target_entity.is_some() {
            // Create the edge entity
            commands.spawn((
                EdgeEntity {
                    edge_id: event.edge_id,
                    graph_id: event.graph_id,
                    source: event.source,
                    target: event.target,
                },
                EdgeRelationship {
                    edge_type: EdgeType::General,
                    label: event.metadata.get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    bidirectional: event.metadata.get("bidirectional")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                },
                EdgeMetadata {
                    properties: event.metadata.clone(),
                    created_at: std::time::SystemTime::now(),
                    updated_at: std::time::SystemTime::now(),
                },
                EdgeWeight::default(),
                EdgeDirection::Forward,
            ));
        }
    }
}

/// System that updates edges from EdgeUpdated events
pub fn update_edge_system(
    mut events: EventReader<EdgeUpdated>,
    mut edge_query: Query<(&EdgeEntity, &mut EdgeRelationship, &mut EdgeMetadata)>,
) {
    for event in events.read() {
        // Find and update the edge
        for (entity, mut relationship, mut metadata) in edge_query.iter_mut() {
            if entity.edge_id == event.edge_id {
                // Update relationship properties
                if let Some(label) = event.metadata.get("label").and_then(|v| v.as_str()) {
                    relationship.label = label.to_string();
                }
                
                // Update metadata
                for (key, value) in &event.metadata {
                    metadata.properties.insert(key.clone(), value.clone());
                }
                metadata.updated_at = std::time::SystemTime::now();
            }
        }
    }
}

/// System that disconnects nodes from EdgeRemoved events
pub fn disconnect_nodes_system(
    mut commands: Commands,
    mut events: EventReader<EdgeRemoved>,
    edge_query: Query<(Entity, &EdgeEntity)>,
) {
    for event in events.read() {
        // Find and remove the edge entity
        for (entity, edge) in edge_query.iter() {
            if edge.edge_id == event.edge_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System that processes edge events for logging/debugging
pub fn process_edge_events_system(
    mut edge_added: EventReader<EdgeAdded>,
    mut edge_updated: EventReader<EdgeUpdated>,
    mut edge_removed: EventReader<EdgeRemoved>,
) {
    for event in edge_added.read() {
        tracing::debug!("Edge added: {:?} from {:?} to {:?}", 
            event.edge_id, event.source, event.target);
    }
    
    for event in edge_updated.read() {
        tracing::debug!("Edge updated: {:?}", event.edge_id);
    }
    
    for event in edge_removed.read() {
        tracing::debug!("Edge removed: {:?}", event.edge_id);
    }
} 