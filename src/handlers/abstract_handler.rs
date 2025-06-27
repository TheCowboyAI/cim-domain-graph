//! Abstract graph command handler that works with any graph implementation

use async_trait::async_trait;
use std::sync::Arc;
use cim_domain::CommandEnvelope;
use crate::{
    GraphId, NodeId, EdgeId,
    commands::{GraphCommand, GraphCommandResult, GraphCommandError},
    events::{GraphCreated, NodeAdded, NodeRemoved, EdgeAdded, EdgeRemoved},
    domain_events::GraphDomainEvent,
    aggregate::abstract_graph::AbstractGraph,
    abstraction::{GraphType, NodeData, EdgeData, Position3D},
};

/// Repository trait for abstract graphs
#[async_trait]
pub trait AbstractGraphRepository: Send + Sync {
    /// Load an abstract graph by ID
    async fn load(&self, graph_id: GraphId) -> GraphCommandResult<AbstractGraph>;
    
    /// Save an abstract graph
    async fn save(&self, graph: &AbstractGraph) -> GraphCommandResult<()>;
    
    /// Check if a graph exists
    async fn exists(&self, graph_id: GraphId) -> GraphCommandResult<bool>;
    
    /// Generate a new graph ID
    async fn next_graph_id(&self) -> GraphCommandResult<GraphId>;
    
    /// Generate a new node ID
    async fn next_node_id(&self) -> GraphCommandResult<NodeId>;
    
    /// Generate a new edge ID
    async fn next_edge_id(&self) -> GraphCommandResult<EdgeId>;
}

/// Command handler that works with abstract graphs
pub struct AbstractGraphCommandHandler {
    repository: Arc<dyn AbstractGraphRepository>,
}

impl AbstractGraphCommandHandler {
    /// Create a new abstract graph command handler
    pub fn new(repository: Arc<dyn AbstractGraphRepository>) -> Self {
        Self { repository }
    }
    
    /// Process a graph command
    pub async fn process_graph_command(
        &self,
        command: GraphCommand,
        _envelope: &CommandEnvelope<GraphCommand>,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        match command {
            GraphCommand::CreateGraph { name, description, metadata } => {
                let graph_id = self.repository.next_graph_id().await?;
                let created_at = chrono::Utc::now();
                
                // Validate input
                if name.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand("Graph name cannot be empty".to_string()));
                }
                
                // Determine graph type from metadata
                let graph_type = metadata.get("graph_type")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "context" => Some(GraphType::new_context(graph_id, &name)),
                        "concept" => Some(GraphType::new_concept(graph_id, &name)),
                        "workflow" => Some(GraphType::new_workflow(graph_id, &name)),
                        "ipld" => Some(GraphType::new_ipld(graph_id)),
                        _ => None,
                    })
                    .unwrap_or_else(|| GraphType::new_context(graph_id, &name));
                
                // Create new abstract graph
                let graph = AbstractGraph::new(graph_type);
                
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
            
            GraphCommand::AddNode { graph_id, node_type, metadata } => {
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;
                let node_id = self.repository.next_node_id().await?;
                
                // Validate input
                if node_type.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand("Node type cannot be empty".to_string()));
                }
                
                // Extract position from metadata
                let position = metadata.get("position")
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
                let event = GraphDomainEvent::NodeRemoved(NodeRemoved {
                    graph_id,
                    node_id,
                });
                
                Ok(vec![event])
            }
            
            GraphCommand::AddEdge { graph_id, source_id, target_id, edge_type, metadata } => {
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;
                let edge_id = self.repository.next_edge_id().await?;
                
                // Validate input
                if edge_type.trim().is_empty() {
                    return Err(GraphCommandError::InvalidCommand("Edge type cannot be empty".to_string()));
                }
                
                // Create edge data
                let edge_data = EdgeData {
                    edge_type: edge_type.clone(),
                    metadata: metadata.clone(),
                };
                
                // Add edge to graph
                graph.add_edge(edge_id, source_id, target_id, edge_data)?;
                
                // Save graph
                self.repository.save(&graph).await?;
                
                // Generate event
                let event = GraphDomainEvent::EdgeAdded(EdgeAdded {
                    graph_id,
                    edge_id,
                    source: source_id,
                    target: target_id,
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
                let event = GraphDomainEvent::EdgeRemoved(EdgeRemoved {
                    graph_id,
                    edge_id,
                });
                
                Ok(vec![event])
            }
            
            GraphCommand::ChangeNodeMetadata { graph_id, node_id, new_metadata } => {
                // Load graph
                let mut graph = self.repository.load(graph_id).await?;
                
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
                self.repository.save(&graph).await?;
                
                // Generate events
                let remove_event = GraphDomainEvent::NodeRemoved(NodeRemoved {
                    graph_id,
                    node_id,
                });
                let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: crate::value_objects::Position3D::default(),
                    node_type: old_data.node_type,
                    metadata: new_metadata,
                });
                
                Ok(vec![remove_event, add_event])
            }
        }
    }
}

/// In-memory implementation of abstract graph repository
pub struct InMemoryAbstractGraphRepository {
    graphs: std::sync::Mutex<std::collections::HashMap<GraphId, AbstractGraph>>,
}

impl Default for InMemoryAbstractGraphRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryAbstractGraphRepository {
    /// Create a new in-memory repository
    pub fn new() -> Self {
        Self {
            graphs: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl AbstractGraphRepository for InMemoryAbstractGraphRepository {
    async fn load(&self, graph_id: GraphId) -> GraphCommandResult<AbstractGraph> {
        let graphs = self.graphs.lock().unwrap();
        graphs.get(&graph_id)
            .cloned()
            .ok_or(GraphCommandError::GraphNotFound(graph_id))
    }
    
    async fn save(&self, graph: &AbstractGraph) -> GraphCommandResult<()> {
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

    #[tokio::test]
    async fn test_create_abstract_graph() {
        let repository = Arc::new(InMemoryAbstractGraphRepository::new());
        let handler = AbstractGraphCommandHandler::new(repository);
        
        // Test creating a context graph
        let mut metadata = HashMap::new();
        metadata.insert("graph_type".to_string(), serde_json::Value::String("context".to_string()));
        
        let command = GraphCommand::CreateGraph {
            name: "Test Context Graph".to_string(),
            description: "A test context graph".to_string(),
            metadata,
        };
        
        let events = handler.process_graph_command(
            command,
            &CommandEnvelope::new(GraphCommand::CreateGraph {
                name: "Test Context Graph".to_string(),
                description: "A test context graph".to_string(),
                metadata: HashMap::new(),
            }, "test".to_string())
        ).await.unwrap();
        
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            GraphDomainEvent::GraphCreated(event) => {
                assert_eq!(event.name, "Test Context Graph");
                assert_eq!(event.description, "A test context graph");
            }
            _ => panic!("Expected GraphCreated event"),
        }
    }

    #[tokio::test]
    async fn test_add_node_to_abstract_graph() {
        let repository = Arc::new(InMemoryAbstractGraphRepository::new());
        let handler = AbstractGraphCommandHandler::new(repository.clone());
        
        // First create a graph
        let mut metadata = HashMap::new();
        metadata.insert("graph_type".to_string(), serde_json::Value::String("workflow".to_string()));
        
        let create_command = GraphCommand::CreateGraph {
            name: "Test Workflow".to_string(),
            description: "A test workflow graph".to_string(),
            metadata,
        };
        
        let create_events = handler.process_graph_command(
            create_command.clone(),
            &CommandEnvelope::new(create_command, "test".to_string())
        ).await.unwrap();
        
        let graph_id = match &create_events[0] {
            GraphDomainEvent::GraphCreated(event) => event.graph_id,
            _ => panic!("Expected GraphCreated event"),
        };
        
        // Now add a node with position
        let mut node_metadata = HashMap::new();
        let position = serde_json::json!({
            "x": 10.0,
            "y": 20.0,
            "z": 0.0
        });
        node_metadata.insert("position".to_string(), position);
        node_metadata.insert("step_type".to_string(), serde_json::Value::String("manual".to_string()));
        
        let add_node_command = GraphCommand::AddNode {
            graph_id,
            node_type: "manual_step".to_string(),
            metadata: node_metadata,
        };
        
        let events = handler.process_graph_command(
            add_node_command.clone(),
            &CommandEnvelope::new(add_node_command, "test".to_string())
        ).await.unwrap();
        
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            GraphDomainEvent::NodeAdded(event) => {
                assert_eq!(event.graph_id, graph_id);
                assert_eq!(event.node_type, "manual_step");
            }
            _ => panic!("Expected NodeAdded event"),
        }
    }
} 