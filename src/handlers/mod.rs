//! Graph command and event handlers
//!
//! Command handlers process graph commands, validate business rules, and emit events.
//! They serve as the bridge between commands and the domain aggregate.

mod abstract_event_handler;
mod abstract_handler;
mod unified_handler;

pub use abstract_event_handler::{AbstractGraphEventHandler, AbstractGraphEventRepository};
pub use abstract_handler::*;
pub use unified_handler::{UnifiedGraphCommandHandler, UnifiedGraphRepository};

use crate::{
    aggregate::Graph,
    commands::{EdgeCommand, GraphCommand, GraphCommandError, GraphCommandResult, NodeCommand},
    domain_events::GraphDomainEvent,
    events::{EdgeAdded, EdgeRemoved, GraphCreated, NodeAdded, NodeRemoved},
    EdgeId, GraphId, NodeId,
};
use async_trait::async_trait;
use cim_domain::{
    AggregateRoot, Command, CommandAcknowledgment, CommandEnvelope, CommandHandler, CommandStatus,
    EntityId,
};
use std::sync::Arc;

/// Trait for handling graph commands
#[async_trait]
pub trait GraphCommandHandler: Send + Sync {
    /// Handle a graph-level command
    async fn handle_graph_command(
        &self,
        command: GraphCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>>;

    /// Handle a node-level command
    async fn handle_node_command(
        &self,
        command: NodeCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>>;

    /// Handle an edge-level command
    async fn handle_edge_command(
        &self,
        command: EdgeCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>>;
}

/// Repository trait for loading and saving graph aggregates
#[async_trait]
pub trait GraphRepository: Send + Sync {
    /// Load a graph aggregate by ID
    async fn load(&self, graph_id: GraphId) -> GraphCommandResult<Graph>;

    /// Save a graph aggregate
    async fn save(&self, graph: &Graph) -> GraphCommandResult<()>;

    /// Check if a graph exists
    async fn exists(&self, graph_id: GraphId) -> GraphCommandResult<bool>;

    /// Generate a new graph ID
    async fn next_graph_id(&self) -> GraphCommandResult<GraphId>;

    /// Generate a new node ID
    async fn next_node_id(&self) -> GraphCommandResult<NodeId>;

    /// Generate a new edge ID
    async fn next_edge_id(&self) -> GraphCommandResult<EdgeId>;
}

/// Implementation of the graph command handler
pub struct GraphCommandHandlerImpl {
    repository: Arc<dyn GraphRepository>,
}

impl GraphCommandHandlerImpl {
    /// Create a new graph command handler
    pub fn new(repository: Arc<dyn GraphRepository>) -> Self {
        Self { repository }
    }

    /// Process a graph command and return events with correlation
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

                // Create new graph aggregate
                let graph = Graph::new(graph_id, name.clone(), description.clone());

                // Save graph
                self.repository.save(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::GraphCreated(GraphCreated {
                    graph_id,
                    name,
                    description,
                    graph_type: None,
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
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;
                let node_id = self.repository.next_node_id().await?;

                // Validate input
                if node_type.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand(
                        "Node type cannot be empty".to_string(),
                    ));
                }

                // Add node to graph
                graph.add_node(node_id, node_type.clone(), metadata.clone())?;

                // Save graph
                self.repository.save(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: crate::value_objects::Position3D::default(),
                    node_type,
                    metadata,
                });

                Ok(vec![event])
            }

            GraphCommand::RemoveNode { graph_id, node_id } => {
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;

                // Remove node from graph
                graph.remove_node(node_id)?;

                // Save graph
                self.repository.save(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::NodeRemoved(NodeRemoved { graph_id, node_id });

                Ok(vec![event])
            }

            GraphCommand::ChangeNodeMetadata {
                graph_id,
                node_id,
                new_metadata,
            } => {
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;

                // Get old node data before removing it
                let old_node = graph
                    .nodes()
                    .get(&node_id)
                    .ok_or(GraphCommandError::NodeNotFound(node_id))?
                    .clone();

                // Change node metadata in graph (remove old, add new)
                graph.change_node_metadata(node_id, new_metadata.clone())?;

                // Save graph
                self.repository.save(&graph).await?;

                // Generate events - remove old node, then add new node with changed metadata
                let remove_event = GraphDomainEvent::NodeRemoved(NodeRemoved { graph_id, node_id });
                let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: crate::value_objects::Position3D::default(),
                    node_type: old_node.node_type,
                    metadata: new_metadata,
                });

                Ok(vec![remove_event, add_event])
            }

            GraphCommand::AddEdge {
                graph_id,
                source_id,
                target_id,
                edge_type,
                metadata,
            } => {
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;
                let edge_id = self.repository.next_edge_id().await?;

                // Validate input
                if edge_type.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand(
                        "Edge type cannot be empty".to_string(),
                    ));
                }

                // Add edge to graph
                graph.add_edge(
                    edge_id,
                    source_id,
                    target_id,
                    edge_type.clone(),
                    metadata.clone(),
                )?;

                // Save graph
                self.repository.save(&graph).await?;

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
                let mut graph = self.repository.load(graph_id).await?;

                // Remove edge from graph
                graph.remove_edge(edge_id)?;

                // Save graph
                self.repository.save(&graph).await?;

                // Generate event
                let event = GraphDomainEvent::EdgeRemoved(EdgeRemoved { graph_id, edge_id });

                Ok(vec![event])
            }
        }
    }
}

// Implement the Command trait for GraphCommand
impl Command for GraphCommand {
    type Aggregate = Graph;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        None // Graph commands don't have a pre-existing aggregate ID for creation
    }
}

// Implement CommandHandler for GraphCommand
impl CommandHandler<GraphCommand> for GraphCommandHandlerImpl {
    fn handle(&mut self, envelope: CommandEnvelope<GraphCommand>) -> CommandAcknowledgment {
        // Extract the command
        let command = envelope.command.clone();
        let command_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();

        // Process the command synchronously (blocking on async)
        let runtime = tokio::runtime::Handle::current();
        let result =
            runtime.block_on(async { self.process_graph_command(command, &envelope).await });

        match result {
            Ok(_events) => {
                // TODO: Publish events with correlation/causation from envelope
                CommandAcknowledgment {
                    command_id,
                    correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            Err(error) => CommandAcknowledgment {
                command_id,
                correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(error.to_string()),
            },
        }
    }
}

// Keep the original GraphCommandHandler implementation for backward compatibility
#[async_trait]
impl GraphCommandHandler for GraphCommandHandlerImpl {
    async fn handle_graph_command(
        &self,
        command: GraphCommand,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        // Create a command envelope for the new flow
        let envelope = CommandEnvelope::new(command.clone(), "graph-handler".to_string());
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

/// In-memory implementation of graph repository for testing
pub struct InMemoryGraphRepository {
    graphs: std::sync::Mutex<std::collections::HashMap<GraphId, Graph>>,
}

impl Default for InMemoryGraphRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryGraphRepository {
    /// Create a new in-memory repository
    pub fn new() -> Self {
        Self {
            graphs: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl GraphRepository for InMemoryGraphRepository {
    async fn load(&self, graph_id: GraphId) -> GraphCommandResult<Graph> {
        let graphs = self.graphs.lock().unwrap();
        graphs
            .get(&graph_id)
            .cloned()
            .ok_or(GraphCommandError::GraphNotFound(graph_id))
    }

    async fn save(&self, graph: &Graph) -> GraphCommandResult<()> {
        let mut graphs = self.graphs.lock().unwrap();
        graphs.insert(graph.id(), graph.clone());
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test Coverage
    ///
    /// ```mermaid
    /// graph TD
    ///     CH[Command Handler] --> GC[Graph Commands]
    ///     CH --> NC[Node Commands]
    ///     CH --> EC[Edge Commands]
    ///     GC --> R[Repository]
    ///     NC --> R
    ///     EC --> R
    ///     R --> A[Aggregate]
    /// ```

    #[tokio::test]
    async fn test_create_graph_command() {
        let repository = Arc::new(InMemoryGraphRepository::new());
        let handler = GraphCommandHandlerImpl::new(repository);

        let command = GraphCommand::CreateGraph {
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            metadata: HashMap::new(),
        };

        let events = handler.handle_graph_command(command).await.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            GraphDomainEvent::GraphCreated(event) => {
                assert_eq!(event.name, "Test Graph");
                assert_eq!(event.description, "A test graph");
            }
            _ => panic!("Expected GraphCreated event"),
        }
    }

    #[tokio::test]
    async fn test_create_graph_validation() {
        let repository = Arc::new(InMemoryGraphRepository::new());
        let handler = GraphCommandHandlerImpl::new(repository);

        let command = GraphCommand::CreateGraph {
            name: "".to_string(), // Empty name should fail
            description: "A test graph".to_string(),
            metadata: HashMap::new(),
        };

        let result = handler.handle_graph_command(command).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphCommandError::InvalidCommand(msg) => {
                assert!(msg.contains("name cannot be empty"));
            }
            _ => panic!("Expected InvalidCommand error"),
        }
    }

    #[tokio::test]
    async fn test_add_node_command() {
        let repository = Arc::new(InMemoryGraphRepository::new());
        let handler = GraphCommandHandlerImpl::new(repository.clone());

        // First create a graph
        let create_command = GraphCommand::CreateGraph {
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            metadata: HashMap::new(),
        };

        let create_events = handler.handle_graph_command(create_command).await.unwrap();
        let graph_id = match &create_events[0] {
            GraphDomainEvent::GraphCreated(event) => event.graph_id,
            _ => panic!("Expected GraphCreated event"),
        };

        // Now add a node
        let add_node_command = GraphCommand::AddNode {
            graph_id,
            node_type: "task".to_string(),
            metadata: HashMap::new(),
        };

        let events = handler
            .handle_graph_command(add_node_command)
            .await
            .unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            GraphDomainEvent::NodeAdded(event) => {
                assert_eq!(event.graph_id, graph_id);
                assert_eq!(event.node_type, "task");
            }
            _ => panic!("Expected NodeAdded event"),
        }
    }

    #[tokio::test]
    async fn test_node_command_delegation() {
        let repository = Arc::new(InMemoryGraphRepository::new());
        let handler = GraphCommandHandlerImpl::new(repository.clone());

        // Create a graph first
        let create_command = GraphCommand::CreateGraph {
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            metadata: HashMap::new(),
        };

        let create_events = handler.handle_graph_command(create_command).await.unwrap();
        let graph_id = match &create_events[0] {
            GraphDomainEvent::GraphCreated(event) => event.graph_id,
            _ => panic!("Expected GraphCreated event"),
        };

        // Test node command delegation
        let node_command = NodeCommand::Add {
            graph_id,
            node_type: "decision".to_string(),
            metadata: HashMap::new(),
        };

        let events = handler.handle_node_command(node_command).await.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            GraphDomainEvent::NodeAdded(event) => {
                assert_eq!(event.node_type, "decision");
            }
            _ => panic!("Expected NodeAdded event"),
        }
    }

    #[tokio::test]
    async fn test_repository_operations() {
        let repository = InMemoryGraphRepository::new();

        // Test ID generation
        let _graph_id = repository.next_graph_id().await.unwrap();
        let _node_id = repository.next_node_id().await.unwrap();
        let _edge_id = repository.next_edge_id().await.unwrap();

        // Test existence check
        let exists = repository.exists(_graph_id).await.unwrap();
        assert!(!exists);

        // Test load non-existent graph
        let load_result = repository.load(_graph_id).await;
        assert!(load_result.is_err());

        match load_result.unwrap_err() {
            GraphCommandError::GraphNotFound(id) => {
                assert_eq!(id, _graph_id);
            }
            _ => panic!("Expected GraphNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let repository = Arc::new(InMemoryGraphRepository::new());
        let handler = GraphCommandHandlerImpl::new(repository);

        // Test adding node to non-existent graph
        let command = GraphCommand::AddNode {
            graph_id: GraphId::new(),
            node_type: "task".to_string(),
            metadata: HashMap::new(),
        };

        let result = handler.handle_graph_command(command).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphCommandError::GraphNotFound(_) => {
                // Expected error
            }
            _ => panic!("Expected GraphNotFound error"),
        }
    }
}
