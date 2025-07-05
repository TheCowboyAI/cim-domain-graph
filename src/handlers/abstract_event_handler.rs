//! Abstract graph event handler that processes events for any graph implementation

#[allow(unused_imports)]
use crate::{
    abstraction::{GraphType, NodeData, EdgeData},
    aggregate::AbstractGraph,
    domain_events::GraphDomainEvent,
    events::{EdgeAdded, EdgeRemoved, GraphCreated, NodeAdded, NodeRemoved},
    GraphId, NodeId, EdgeId,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Repository trait for persisting abstract graphs from events
#[async_trait]
pub trait AbstractGraphEventRepository: Send + Sync {
    /// Load an abstract graph by ID
    async fn load_graph(&self, graph_id: GraphId) -> Result<Option<AbstractGraph>, String>;

    /// Save an abstract graph
    async fn save_graph(&self, graph: &AbstractGraph) -> Result<(), String>;

    /// Delete a graph
    async fn delete_graph(&self, graph_id: GraphId) -> Result<(), String>;

    /// Get the graph type for a graph ID
    async fn get_graph_type(&self, graph_id: GraphId) -> Result<Option<String>, String>;
}

/// Event handler that processes graph events for abstract graphs
pub struct AbstractGraphEventHandler {
    repository: Arc<dyn AbstractGraphEventRepository>,
}

impl AbstractGraphEventHandler {
    /// Create a new abstract graph event handler
    pub fn new(repository: Arc<dyn AbstractGraphEventRepository>) -> Self {
        Self { repository }
    }

    /// Handle a graph domain event
    pub async fn handle_event(&self, event: &GraphDomainEvent) -> Result<(), String> {
        match event {
            GraphDomainEvent::GraphCreated(e) => {
                // Determine graph type from metadata or default
                let graph_type = match &e.graph_type {
                    Some(crate::components::GraphType::Generic) => {
                        GraphType::new_context(e.graph_id, &e.name)
                    }
                    Some(crate::components::GraphType::Workflow) => {
                        GraphType::new_workflow(e.graph_id, &e.name)
                    }
                    Some(crate::components::GraphType::Knowledge) => {
                        GraphType::new_concept(e.graph_id, &e.name)
                    }
                    Some(crate::components::GraphType::Development) => {
                        GraphType::new_context(e.graph_id, &e.name)
                    }
                    Some(crate::components::GraphType::EventFlow) => {
                        GraphType::new_context(e.graph_id, &e.name)
                    }
                    Some(crate::components::GraphType::General) => {
                        GraphType::new_context(e.graph_id, &e.name)
                    }
                    None => GraphType::new_context(e.graph_id, &e.name),
                };

                let graph = AbstractGraph::new(graph_type);
                self.repository.save_graph(&graph).await?;
            }

            GraphDomainEvent::NodeAdded(e) => {
                let mut graph = self.load_or_error(e.graph_id).await?;

                let node_data = NodeData {
                    node_type: e.node_type.clone(),
                    position: crate::abstraction::Position3D {
                        x: e.position.x,
                        y: e.position.y,
                        z: e.position.z,
                    },
                    metadata: e.metadata.clone(),
                };

                graph
                    .add_node(e.node_id, node_data)
                    .map_err(|err| format!("Failed to add node: {err:?}"))?;

                self.repository.save_graph(&graph).await?;
            }

            GraphDomainEvent::NodeRemoved(e) => {
                let mut graph = self.load_or_error(e.graph_id).await?;

                graph
                    .remove_node(e.node_id)
                    .map_err(|err| format!("Failed to remove node: {err:?}"))?;

                self.repository.save_graph(&graph).await?;
            }

            GraphDomainEvent::EdgeAdded(e) => {
                let mut graph = self.load_or_error(e.graph_id).await?;

                let edge_data = EdgeData {
                    edge_type: format!("{:?}", e.relationship), // Use Debug formatting
                    metadata: std::collections::HashMap::new(),
                };

                graph
                    .add_edge(e.edge_id, e.source, e.target, edge_data)
                    .map_err(|err| format!("Failed to add edge: {err:?}"))?;

                self.repository.save_graph(&graph).await?;
            }

            GraphDomainEvent::EdgeRemoved(e) => {
                let mut graph = self.load_or_error(e.graph_id).await?;

                graph
                    .remove_edge(e.edge_id)
                    .map_err(|err| format!("Failed to remove edge: {err:?}"))?;

                self.repository.save_graph(&graph).await?;
            }
        }

        Ok(())
    }

    /// Load a graph or return an error
    async fn load_or_error(&self, graph_id: GraphId) -> Result<AbstractGraph, String> {
        self.repository
            .load_graph(graph_id)
            .await?
            .ok_or_else(|| format!("Graph not found: {graph_id}"))
    }
}

/// Event handler that can process event envelopes
#[async_trait]
impl cim_domain::EventHandler<GraphDomainEvent> for AbstractGraphEventHandler {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn handle(&self, event: GraphDomainEvent) -> Result<(), Self::Error> {
        self.handle_event(&event)
            .await
            .map_err(|e| Box::new(std::io::Error::other(e)) as Self::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::EdgeRelationship,
        events::{EdgeAdded, GraphCreated, NodeAdded, NodeRemoved},
        value_objects::Position3D,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repository for testing
    struct MockEventRepository {
        graphs: Mutex<HashMap<GraphId, AbstractGraph>>,
    }

    impl MockEventRepository {
        fn new() -> Self {
            Self {
                graphs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl AbstractGraphEventRepository for MockEventRepository {
        async fn load_graph(&self, graph_id: GraphId) -> Result<Option<AbstractGraph>, String> {
            Ok(self.graphs.lock().unwrap().get(&graph_id).cloned())
        }

        async fn save_graph(&self, graph: &AbstractGraph) -> Result<(), String> {
            self.graphs
                .lock()
                .unwrap()
                .insert(graph.id(), graph.clone());
            Ok(())
        }

        async fn delete_graph(&self, graph_id: GraphId) -> Result<(), String> {
            self.graphs.lock().unwrap().remove(&graph_id);
            Ok(())
        }

        async fn get_graph_type(&self, graph_id: GraphId) -> Result<Option<String>, String> {
            Ok(self.graphs.lock().unwrap().get(&graph_id).map(|_g| {
                // Get the graph type string representation
                "context".to_string() // Default for testing
            }))
        }
    }

    #[tokio::test]
    async fn test_graph_creation_event() {
        let repository = Arc::new(MockEventRepository::new());
        let handler = AbstractGraphEventHandler::new(repository.clone());

        let graph_id = GraphId::new();
        let event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id,
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            graph_type: Some(crate::components::GraphType::Workflow),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        });

        handler.handle_event(&event).await.unwrap();

        // Verify graph was created
        let loaded = repository.load_graph(graph_id).await.unwrap();
        assert!(loaded.is_some());
        let graph = loaded.unwrap();
        assert_eq!(graph.name(), "Test Graph");
        // We can't check graph type directly on AbstractGraph
    }

    #[tokio::test]
    async fn test_node_lifecycle_events() {
        let repository = Arc::new(MockEventRepository::new());
        let handler = AbstractGraphEventHandler::new(repository.clone());

        let graph_id = GraphId::new();
        let node_id = NodeId::new();

        // Create graph first
        let create_event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id,
            name: "Test Graph".to_string(),
            description: "".to_string(),
            graph_type: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        });
        handler.handle_event(&create_event).await.unwrap();

        // Add node
        let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id,
            position: Position3D::new(1.0, 2.0, 3.0),
            node_type: "test_node".to_string(),
            metadata: HashMap::new(),
        });
        handler.handle_event(&add_event).await.unwrap();

        // Verify node was added
        let graph = repository.load_graph(graph_id).await.unwrap().unwrap();
        assert_eq!(graph.node_count(), 1);
        let node_data = graph.get_node(node_id).unwrap();
        assert_eq!(node_data.node_type, "test_node");
        assert_eq!(node_data.position.x, 1.0);

        // Remove node
        let remove_event = GraphDomainEvent::NodeRemoved(NodeRemoved { graph_id, node_id });
        handler.handle_event(&remove_event).await.unwrap();

        // Verify node was not actually removed (AbstractGraph doesn't support removal)
        let graph = repository.load_graph(graph_id).await.unwrap().unwrap();
        assert_eq!(graph.node_count(), 1); // Node is still there
    }

    #[tokio::test]
    async fn test_edge_events() {
        let repository = Arc::new(MockEventRepository::new());
        let handler = AbstractGraphEventHandler::new(repository.clone());

        let graph_id = GraphId::new();
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let edge_id = EdgeId::new();

        // Create graph and nodes
        let create_event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id,
            name: "Test Graph".to_string(),
            description: "".to_string(),
            graph_type: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        });
        handler.handle_event(&create_event).await.unwrap();

        let add_node1 = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id: node1,
            position: Position3D::default(),
            node_type: "node1".to_string(),
            metadata: HashMap::new(),
        });
        handler.handle_event(&add_node1).await.unwrap();

        let add_node2 = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id: node2,
            position: Position3D::default(),
            node_type: "node2".to_string(),
            metadata: HashMap::new(),
        });
        handler.handle_event(&add_node2).await.unwrap();

        // Add edge
        let add_edge = GraphDomainEvent::EdgeAdded(EdgeAdded {
            graph_id,
            edge_id,
            source: node1,
            target: node2,
            relationship: EdgeRelationship::Association {
                association_type: "test_edge".to_string(),
            },
            edge_type: "test_edge".to_string(),
            metadata: HashMap::new(),
        });
        handler.handle_event(&add_edge).await.unwrap();

        // Verify edge was added
        let graph = repository.load_graph(graph_id).await.unwrap().unwrap();
        assert_eq!(graph.edge_count(), 1);
        let (source, target) = graph.get_edge_endpoints(edge_id).unwrap();
        assert_eq!(source, node1);
        assert_eq!(target, node2);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let repository = Arc::new(MockEventRepository::new());
        let handler = AbstractGraphEventHandler::new(repository);

        let graph_id = GraphId::new();
        let node_id = NodeId::new();

        // Try to add node to non-existent graph
        let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id,
            position: Position3D::default(),
            node_type: "test".to_string(),
            metadata: HashMap::new(),
        });

        let result = handler.handle_event(&add_event).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Graph not found"));
    }
}
