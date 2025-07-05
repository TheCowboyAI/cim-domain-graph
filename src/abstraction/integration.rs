//! Integration module for bridging existing systems with the graph abstraction layer

use super::{
    GraphType as AbstractGraphType, GraphImplementation, NodeData, EdgeData,
    GraphOperationError,
};
use crate::{
    components::*, 
    events::*,
    handlers::UnifiedGraphCommandHandler,
    queries::AbstractGraphQueryHandler,
    GraphId,
};
use bevy_ecs::prelude::*;
use bevy_app::{App, Plugin, Update};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resource that provides access to the graph abstraction layer
#[derive(Resource)]
pub struct GraphAbstractionLayer {
    /// Handler for unified graph operations
    pub command_handler: Arc<UnifiedGraphCommandHandler>,
    
    /// Handler for abstract graph queries
    pub query_handler: Arc<AbstractGraphQueryHandler>,
    
    /// Cache of active abstract graphs
    graphs: Arc<RwLock<std::collections::HashMap<GraphId, AbstractGraphType>>>,
}

impl GraphAbstractionLayer {
    /// Create a new graph abstraction layer
    pub fn new(
        command_handler: Arc<UnifiedGraphCommandHandler>,
        query_handler: Arc<AbstractGraphQueryHandler>,
    ) -> Self {
        Self {
            command_handler,
            query_handler,
            graphs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Get or create an abstract graph for the given graph ID and type
    pub async fn get_or_create_graph(
        &self,
        graph_id: GraphId,
        graph_type: GraphType,
        name: &str,
    ) -> Result<AbstractGraphType, GraphOperationError> {
        let mut graphs = self.graphs.write().await;
        
        if let Some(graph) = graphs.get(&graph_id) {
            Ok(graph.clone())
        } else {
            let abstract_graph = match graph_type {
                GraphType::Workflow => AbstractGraphType::new_workflow(graph_id, name),
                GraphType::Knowledge => AbstractGraphType::new_concept(graph_id, name),
                GraphType::Development => AbstractGraphType::new_context(graph_id, name),
                GraphType::EventFlow => AbstractGraphType::new_context(graph_id, name),
                _ => AbstractGraphType::new_context(graph_id, name), // Default to context
            };
            
            graphs.insert(graph_id, abstract_graph.clone());
            Ok(abstract_graph)
        }
    }
    
    /// Remove a graph from the cache
    pub async fn remove_graph(&self, graph_id: GraphId) {
        let mut graphs = self.graphs.write().await;
        graphs.remove(&graph_id);
    }
}

/// System that integrates graph creation with the abstraction layer
pub fn integrated_create_graph_system(
    mut commands: Commands,
    mut events: EventReader<GraphCreated>,
    abstraction: Res<GraphAbstractionLayer>,
) {
    for event in events.read() {
        // Create the graph entity as before
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
        
        // Also create the abstract graph representation
        let graph_id = event.graph_id;
        let graph_type = event.graph_type.unwrap_or(GraphType::Generic);
        let name = event.name.clone();
        let graphs = abstraction.graphs.clone();
        
        // Spawn async task to create abstract graph
        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(async move {
            let mut graphs_lock = graphs.write().await;
            
            graphs_lock.entry(graph_id).or_insert_with(|| {
                let abstract_graph = match graph_type {
                    GraphType::Workflow => AbstractGraphType::new_workflow(graph_id, &name),
                    GraphType::Knowledge => AbstractGraphType::new_concept(graph_id, &name),
                    GraphType::Development => AbstractGraphType::new_context(graph_id, &name),
                    GraphType::EventFlow => AbstractGraphType::new_context(graph_id, &name),
                    _ => AbstractGraphType::new_context(graph_id, &name),
                };
                
                abstract_graph
            });
        });
    }
}

/// System that integrates node operations with the abstraction layer
pub fn integrated_node_system(
    mut events: EventReader<NodeAdded>,
    abstraction: Res<GraphAbstractionLayer>,
    query: Query<&NodeEntity>,
) {
    for event in events.read() {
        // Get node data from ECS
        if let Some(_node) = query.iter().find(|n| n.node_id == event.node_id) {
            let graph_id = event.graph_id;
            let node_id = event.node_id;
            let node_data = NodeData {
                node_type: event.node_type.clone(),
                position: event.position,
                metadata: event.metadata.clone(),
            };
            
            let graphs = abstraction.graphs.clone();
            
            // Update abstract graph
            #[cfg(not(target_arch = "wasm32"))]
            tokio::spawn(async move {
                let mut graphs_lock = graphs.write().await;
                
                // Get or create a generic graph for this graph_id
                let graph = graphs_lock.entry(graph_id).or_insert_with(|| {
                    AbstractGraphType::new_context(graph_id, "Unknown")
                });
                
                if let Err(e) = graph.add_node(node_id, node_data) {
                    tracing::error!("Failed to add node to abstract graph: {:?}", e);
                }
            });
        }
    }
}

/// System that integrates edge operations with the abstraction layer
pub fn integrated_edge_system(
    mut events: EventReader<EdgeAdded>,
    abstraction: Res<GraphAbstractionLayer>,
    query: Query<&EdgeEntity>,
) {
    for event in events.read() {
        // Get edge data from ECS
        if let Some(_edge) = query.iter().find(|e| e.edge_id == event.edge_id) {
            let graph_id = event.graph_id;
            let edge_id = event.edge_id;
            let source = event.source;
            let target = event.target;
            let edge_data = EdgeData {
                edge_type: event.edge_type.clone(),
                metadata: event.metadata.clone(),
            };
            
            let graphs = abstraction.graphs.clone();
            
            // Update abstract graph
            #[cfg(not(target_arch = "wasm32"))]
            tokio::spawn(async move {
                let mut graphs_lock = graphs.write().await;
                
                // Get or create a generic graph for this graph_id
                let graph = graphs_lock.entry(graph_id).or_insert_with(|| {
                    AbstractGraphType::new_context(graph_id, "Unknown")
                });
                
                if let Err(e) = graph.add_edge(edge_id, source, target, edge_data) {
                    tracing::error!("Failed to add edge to abstract graph: {:?}", e);
                }
            });
        }
    }
}

/// Plugin that integrates the graph abstraction layer with Bevy systems
pub struct GraphAbstractionPlugin {
    pub command_handler: Arc<UnifiedGraphCommandHandler>,
    pub query_handler: Arc<AbstractGraphQueryHandler>,
}

impl Plugin for GraphAbstractionPlugin {
    fn build(&self, app: &mut App) {
        // Insert the abstraction layer as a resource
        app.insert_resource(GraphAbstractionLayer::new(
            self.command_handler.clone(),
            self.query_handler.clone(),
        ));
        
        // Replace existing systems with integrated versions
        app.add_systems(
            Update,
            (
                integrated_create_graph_system,
                integrated_node_system,
                integrated_edge_system,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_graph_abstraction_layer_basic() {
        // Test basic graph caching functionality
        let graphs = Arc::new(RwLock::new(std::collections::HashMap::new()));
        
        // Add a graph
        let graph_id = GraphId::new();
        let graph = AbstractGraphType::new_context(graph_id, "Test Graph");
        
        {
            let mut graphs_lock = graphs.write().await;
            graphs_lock.insert(graph_id, graph.clone());
        }
        
        // Verify it's cached
        {
            let graphs_lock = graphs.read().await;
            assert!(graphs_lock.contains_key(&graph_id));
        }
        
        // Remove it
        {
            let mut graphs_lock = graphs.write().await;
            graphs_lock.remove(&graph_id);
        }
        
        // Verify it's gone
        {
            let graphs_lock = graphs.read().await;
            assert!(!graphs_lock.contains_key(&graph_id));
        }
    }
} 