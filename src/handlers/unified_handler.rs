//! Unified graph command handler that supports all graph types through the abstract interface

use crate::{
    abstraction::{EdgeData, GraphType, NodeData, Position3D},
    aggregate::abstract_graph::AbstractGraph,
    commands::{EdgeCommand, GraphCommand, GraphCommandError, GraphCommandResult, NodeCommand},
    domain_events::GraphDomainEvent,
    events::{EdgeAdded, EdgeRemoved, GraphCreated, NodeAdded, NodeRemoved},
    handlers::GraphCommandHandler,
    EdgeId, GraphId, NodeId,
};
use async_trait::async_trait;
use cim_domain::{CommandAcknowledgment, CommandEnvelope, CommandHandler, CommandStatus};
use std::sync::Arc;

/// Unified repository that can handle both concrete and abstract graphs
#[async_trait]
pub trait UnifiedGraphRepository: Send + Sync {
    /// Load a graph by ID and type
    async fn load_graph(
        &self,
        graph_id: GraphId,
        graph_type: Option<&str>,
    ) -> GraphCommandResult<AbstractGraph>;

    /// Save a graph
    async fn save_graph(&self, graph: &AbstractGraph) -> GraphCommandResult<()>;

    /// Check if a graph exists
    async fn exists(&self, graph_id: GraphId) -> GraphCommandResult<bool>;

    /// Generate a new graph ID
    async fn next_graph_id(&self) -> GraphCommandResult<GraphId>;

    /// Generate a new node ID
    async fn next_node_id(&self) -> GraphCommandResult<NodeId>;

    /// Generate a new edge ID
    async fn next_edge_id(&self) -> GraphCommandResult<EdgeId>;

    /// Get graph type for an existing graph
    async fn get_graph_type(&self, graph_id: GraphId) -> GraphCommandResult<Option<String>>;
}

/// Unified graph command handler that works with all graph types
pub struct UnifiedGraphCommandHandler {
    repository: Arc<dyn UnifiedGraphRepository>,
}

impl UnifiedGraphCommandHandler {
    /// Create a new unified graph command handler
    pub fn new(repository: Arc<dyn UnifiedGraphRepository>) -> Self {
        Self { repository }
    }

    /// Determine graph type from metadata or existing graph
    async fn determine_graph_type(
        &self,
        graph_id: Option<GraphId>,
        metadata: &std::collections::HashMap<String, serde_json::Value>,
    ) -> GraphCommandResult<String> {
        // First check metadata
        if let Some(graph_type) = metadata
            .get("graph_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            return Ok(graph_type);
        }

        // If we have a graph_id, check existing graph
        if let Some(id) = graph_id {
            if let Ok(Some(graph_type)) = self.repository.get_graph_type(id).await {
                return Ok(graph_type);
            }
        }

        // Default to context graph
        Ok("context".to_string())
    }

    /// Process a graph command
    async fn process_graph_command(
        &self,
        command: GraphCommand,
        _envelope: &CommandEnvelope<GraphCommand>,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        match command {
            GraphCommand::CreateGraph {
                name,
                description,
                metadata,
            } => {
                let graph_id = self.repository.next_graph_id().await?;
                let created_at = chrono::Utc::now();

                // Validate input
                if name.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand(
                        "Graph name cannot be empty".to_string(),
                    ));
                }

                // Determine graph type
                let graph_type_str = self.determine_graph_type(None, &metadata).await?;

                // Create appropriate graph type
                let graph_type = match graph_type_str.as_str() {
                    "context" => GraphType::new_context(graph_id, &name),
                    "concept" => GraphType::new_concept(graph_id, &name),
                    "workflow" => GraphType::new_workflow(graph_id, &name),
                    "ipld" => GraphType::new_ipld(graph_id),
                    _ => GraphType::new_context(graph_id, &name), // Default
                };

                // Create new abstract graph
                let graph = AbstractGraph::new(graph_type);

                // Save graph
                self.repository.save_graph(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::GraphCreated(GraphCreated {
                    graph_id,
                    name,
                    description,
                    graph_type: Some(crate::components::GraphType::Generic), // Map to component type
                    metadata,
                    created_at,
                });

                Ok(vec![event])
            }

            GraphCommand::AddNode {
                graph_id,
                node_type,
                metadata,
            } => {
                // Load graph with type information
                let graph_type_str = self.determine_graph_type(Some(graph_id), &metadata).await?;
                let mut graph = self
                    .repository
                    .load_graph(graph_id, Some(&graph_type_str))
                    .await?;
                let node_id = self.repository.next_node_id().await?;

                // Validate input
                if node_type.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand(
                        "Node type cannot be empty".to_string(),
                    ));
                }

                // Extract position from metadata
                let position = metadata
                    .get("position")
                    .and_then(|v| v.as_object())
                    .map(|obj| Position3D {
                        x: obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        y: obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        z: obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    })
                    .unwrap_or_default();

                // Create node data
                let node_data = NodeData {
                    node_type: node_type.clone(),
                    position,
                    metadata: metadata.clone(),
                };

                // Add node to graph
                graph.add_node(node_id, node_data)?;

                // Save graph
                self.repository.save_graph(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: crate::value_objects::Position3D::new(
                        position.x, position.y, position.z,
                    ),
                    node_type,
                    metadata,
                });

                Ok(vec![event])
            }

            GraphCommand::RemoveNode { graph_id, node_id } => {
                // Load graph
                let graph_type_str = self
                    .determine_graph_type(Some(graph_id), &std::collections::HashMap::new())
                    .await?;
                let mut graph = self
                    .repository
                    .load_graph(graph_id, Some(&graph_type_str))
                    .await?;

                // Remove node from graph
                graph.remove_node(node_id)?;

                // Save graph
                self.repository.save_graph(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::NodeRemoved(NodeRemoved { graph_id, node_id });

                Ok(vec![event])
            }

            GraphCommand::AddEdge {
                graph_id,
                source_id,
                target_id,
                edge_type,
                metadata,
            } => {
                // Load graph
                let graph_type_str = self.determine_graph_type(Some(graph_id), &metadata).await?;
                let mut graph = self
                    .repository
                    .load_graph(graph_id, Some(&graph_type_str))
                    .await?;
                let edge_id = self.repository.next_edge_id().await?;

                // Validate input
                if edge_type.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand(
                        "Edge type cannot be empty".to_string(),
                    ));
                }

                // Create edge data
                let edge_data = EdgeData {
                    edge_type: edge_type.clone(),
                    metadata: metadata.clone(),
                };

                // Add edge to graph
                graph.add_edge(edge_id, source_id, target_id, edge_data)?;

                // Save graph
                self.repository.save_graph(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::EdgeAdded(EdgeAdded {
                    graph_id,
                    edge_id,
                    source: source_id,
                    target: target_id,
                    relationship: crate::components::EdgeRelationship::Association {
                        association_type: edge_type.clone(),
                    },
                    edge_type,
                    metadata,
                });

                Ok(vec![event])
            }

            GraphCommand::RemoveEdge { graph_id, edge_id } => {
                // Load graph
                let graph_type_str = self
                    .determine_graph_type(Some(graph_id), &std::collections::HashMap::new())
                    .await?;
                let mut graph = self
                    .repository
                    .load_graph(graph_id, Some(&graph_type_str))
                    .await?;

                // Remove edge from graph
                graph.remove_edge(edge_id)?;

                // Save graph
                self.repository.save_graph(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::EdgeRemoved(EdgeRemoved { graph_id, edge_id });

                Ok(vec![event])
            }

            GraphCommand::ChangeNodeMetadata {
                graph_id,
                node_id,
                new_metadata,
            } => {
                // Load graph
                let graph_type_str = self
                    .determine_graph_type(Some(graph_id), &new_metadata)
                    .await?;
                let mut graph = self
                    .repository
                    .load_graph(graph_id, Some(&graph_type_str))
                    .await?;

                // Get old node data
                let old_data = graph.get_node(node_id)?;

                // Create new node data with updated metadata
                let new_data = NodeData {
                    node_type: old_data.node_type.clone(),
                    position: old_data.position,
                    metadata: new_metadata.clone(),
                };

                // Update node (remove old, add new)
                graph.remove_node(node_id)?;
                graph.add_node(node_id, new_data)?;

                // Save graph
                self.repository.save_graph(&graph).await?;

                // Generate events
                let remove_event = GraphDomainEvent::NodeRemoved(NodeRemoved { graph_id, node_id });
                let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: crate::value_objects::Position3D::new(
                        old_data.position.x,
                        old_data.position.y,
                        old_data.position.z,
                    ),
                    node_type: old_data.node_type,
                    metadata: new_metadata,
                });

                Ok(vec![remove_event, add_event])
            }
        }
    }
}

// Implement CommandHandler for unified handler
impl CommandHandler<GraphCommand> for UnifiedGraphCommandHandler {
    fn handle(&mut self, envelope: CommandEnvelope<GraphCommand>) -> CommandAcknowledgment {
        let command = envelope.command.clone();
        let command_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();

        // Process the command synchronously (blocking on async)
        let runtime = tokio::runtime::Handle::current();
        let result =
            runtime.block_on(async { self.process_graph_command(command, &envelope).await });

        match result {
            Ok(_events) => CommandAcknowledgment {
                command_id,
                correlation_id,
                status: CommandStatus::Accepted,
                reason: None,
            },
            Err(error) => CommandAcknowledgment {
                command_id,
                correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(error.to_string()),
            },
        }
    }
}

// Implement GraphCommandHandler trait for compatibility
#[async_trait]
impl GraphCommandHandler for UnifiedGraphCommandHandler {
    async fn handle_graph_command(
        &self,
        command: GraphCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        let envelope = CommandEnvelope::new(command.clone(), "unified-handler".to_string());
        self.process_graph_command(command, &envelope).await
    }

    async fn handle_node_command(
        &self,
        command: NodeCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        match command {
            NodeCommand::Add {
                graph_id,
                node_type,
                metadata,
            } => {
                let graph_command = GraphCommand::AddNode {
                    graph_id,
                    node_type,
                    metadata,
                };
                self.handle_graph_command(graph_command).await
            }

            NodeCommand::Remove { graph_id, node_id } => {
                let graph_command = GraphCommand::RemoveNode { graph_id, node_id };
                self.handle_graph_command(graph_command).await
            }

            NodeCommand::ChangeMetadata {
                graph_id,
                node_id,
                new_metadata,
            } => {
                let graph_command = GraphCommand::ChangeNodeMetadata {
                    graph_id,
                    node_id,
                    new_metadata,
                };
                self.handle_graph_command(graph_command).await
            }
        }
    }

    async fn handle_edge_command(
        &self,
        command: EdgeCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        match command {
            EdgeCommand::Add {
                graph_id,
                source_id,
                target_id,
                edge_type,
                metadata,
            } => {
                let graph_command = GraphCommand::AddEdge {
                    graph_id,
                    source_id,
                    target_id,
                    edge_type,
                    metadata,
                };
                self.handle_graph_command(graph_command).await
            }

            EdgeCommand::Remove { graph_id, edge_id } => {
                let graph_command = GraphCommand::RemoveEdge { graph_id, edge_id };
                self.handle_graph_command(graph_command).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock implementation of UnifiedGraphRepository for testing
    struct MockUnifiedRepository {
        graphs: std::sync::Mutex<HashMap<GraphId, (AbstractGraph, String)>>,
    }

    impl MockUnifiedRepository {
        fn new() -> Self {
            Self {
                graphs: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UnifiedGraphRepository for MockUnifiedRepository {
        async fn load_graph(
            &self,
            graph_id: GraphId,
            _graph_type: Option<&str>,
        ) -> GraphCommandResult<AbstractGraph> {
            let graphs = self.graphs.lock().unwrap();
            graphs
                .get(&graph_id)
                .map(|(graph, _)| graph.clone())
                .ok_or(GraphCommandError::GraphNotFound(graph_id))
        }

        async fn save_graph(&self, graph: &AbstractGraph) -> GraphCommandResult<()> {
            let mut graphs = self.graphs.lock().unwrap();
            let graph_type = match &graph.graph {
                GraphType::Context(_) => "context",
                GraphType::Concept(_) => "concept",
                GraphType::Workflow(_) => "workflow",
                GraphType::Ipld(_) => "ipld",
            };
            graphs.insert(graph.id(), (graph.clone(), graph_type.to_string()));
            Ok(())
        }

        async fn exists(&self, graph_id: GraphId) -> GraphCommandResult<bool> {
            let graphs = self.graphs.lock().unwrap();
            Ok(graphs.contains_key(&graph_id))
        }

        async fn next_graph_id(&self) -> GraphCommandResult<GraphId> {
            Ok(GraphId::new())
        }

        async fn next_node_id(&self) -> GraphCommandResult<NodeId> {
            Ok(NodeId::new())
        }

        async fn next_edge_id(&self) -> GraphCommandResult<EdgeId> {
            Ok(EdgeId::new())
        }

        async fn get_graph_type(&self, graph_id: GraphId) -> GraphCommandResult<Option<String>> {
            let graphs = self.graphs.lock().unwrap();
            Ok(graphs.get(&graph_id).map(|(_, t)| t.clone()))
        }
    }

    #[tokio::test]
    async fn test_unified_handler_create_graph() {
        let repository = Arc::new(MockUnifiedRepository::new());
        let handler = UnifiedGraphCommandHandler::new(repository);

        // Test creating different graph types
        for (graph_type, name) in [
            ("context", "Context Graph"),
            ("concept", "Concept Graph"),
            ("workflow", "Workflow Graph"),
            ("ipld", "IPLD Graph"),
        ] {
            let mut metadata = HashMap::new();
            metadata.insert(
                "graph_type".to_string(),
                serde_json::Value::String(graph_type.to_string()),
            );

            let command = GraphCommand::CreateGraph {
                name: name.to_string(),
                description: format!("A test {graph_type} graph"),
                metadata,
            };

            let events = handler.handle_graph_command(command).await.unwrap();
            assert_eq!(events.len(), 1);

            match &events[0] {
                GraphDomainEvent::GraphCreated(event) => {
                    assert_eq!(event.name, name);
                }
                _ => panic!("Expected GraphCreated event"),
            }
        }
    }

    #[tokio::test]
    async fn test_unified_handler_with_nodes_and_edges() {
        let repository = Arc::new(MockUnifiedRepository::new());
        let handler = UnifiedGraphCommandHandler::new(repository);

        // Create a workflow graph
        let mut metadata = HashMap::new();
        metadata.insert(
            "graph_type".to_string(),
            serde_json::Value::String("workflow".to_string()),
        );

        let create_command = GraphCommand::CreateGraph {
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            metadata,
        };

        let create_events = handler.handle_graph_command(create_command).await.unwrap();
        let graph_id = match &create_events[0] {
            GraphDomainEvent::GraphCreated(event) => event.graph_id,
            _ => panic!("Expected GraphCreated event"),
        };

        // Add nodes
        let add_node_command = GraphCommand::AddNode {
            graph_id,
            node_type: "start".to_string(),
            metadata: HashMap::new(),
        };

        let node_events = handler
            .handle_graph_command(add_node_command)
            .await
            .unwrap();
        assert_eq!(node_events.len(), 1);

        // Add another node
        let add_node2_command = GraphCommand::AddNode {
            graph_id,
            node_type: "end".to_string(),
            metadata: HashMap::new(),
        };

        let node2_events = handler
            .handle_graph_command(add_node2_command)
            .await
            .unwrap();

        let node1_id = match &node_events[0] {
            GraphDomainEvent::NodeAdded(event) => event.node_id,
            _ => panic!("Expected NodeAdded event"),
        };

        let node2_id = match &node2_events[0] {
            GraphDomainEvent::NodeAdded(event) => event.node_id,
            _ => panic!("Expected NodeAdded event"),
        };

        // Connect nodes with edge
        let add_edge_command = GraphCommand::AddEdge {
            graph_id,
            source_id: node1_id,
            target_id: node2_id,
            edge_type: "sequence".to_string(),
            metadata: HashMap::new(),
        };

        let edge_events = handler
            .handle_graph_command(add_edge_command)
            .await
            .unwrap();
        assert_eq!(edge_events.len(), 1);

        match &edge_events[0] {
            GraphDomainEvent::EdgeAdded(event) => {
                assert_eq!(event.source, node1_id);
                assert_eq!(event.target, node2_id);
                assert_eq!(event.edge_type, "sequence");
            }
            _ => panic!("Expected EdgeAdded event"),
        }
    }
}
