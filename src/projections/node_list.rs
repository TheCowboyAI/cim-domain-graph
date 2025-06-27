//! Node list projection
//!
//! Provides a searchable list of all nodes across graphs.

use crate::{
    domain_events::{GraphDomainEvent},
    events::{NodeAdded, NodeRemoved},
    GraphId, NodeId,
};
use cim_domain::projections::{EventSequence, Projection};
use cim_domain::DomainEventEnum;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a node for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique identifier of the node
    pub node_id: NodeId,
    /// ID of the graph this node belongs to
    pub graph_id: GraphId,
    /// Type/category of the node
    pub node_type: String,
    /// Optional human-readable name
    pub name: Option<String>,
    /// Additional metadata about the node
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Projection that maintains a searchable list of nodes
#[derive(Debug, Clone)]
pub struct NodeListProjection {
    nodes: HashMap<NodeId, NodeInfo>,
    nodes_by_graph: HashMap<GraphId, Vec<NodeId>>,
    nodes_by_type: HashMap<String, Vec<NodeId>>,
    checkpoint: Option<EventSequence>,
}

impl Default for NodeListProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeListProjection {
    /// Create a new node list projection
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            nodes_by_graph: HashMap::new(),
            nodes_by_type: HashMap::new(),
            checkpoint: None,
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &NodeId) -> Option<&NodeInfo> {
        self.nodes.get(node_id)
    }

    /// Get all nodes
    pub fn get_all_nodes(&self) -> Vec<&NodeInfo> {
        self.nodes.values().collect()
    }

    /// Get nodes by graph ID
    pub fn get_nodes_by_graph(&self, graph_id: &GraphId) -> Vec<&NodeInfo> {
        self.nodes_by_graph
            .get(graph_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get nodes by type
    pub fn get_nodes_by_type(&self, node_type: &str) -> Vec<&NodeInfo> {
        self.nodes_by_type
            .get(node_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Search nodes by name (case-insensitive partial match)
    pub fn search_by_name(&self, query: &str) -> Vec<&NodeInfo> {
        let query_lower = query.to_lowercase();
        self.nodes
            .values()
            .filter(|node| {
                node.name
                    .as_ref()
                    .map(|name| name.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get total number of nodes
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get node count by type
    pub fn count_by_type(&self) -> HashMap<String, usize> {
        self.nodes_by_type
            .iter()
            .map(|(node_type, ids)| (node_type.clone(), ids.len()))
            .collect()
    }
}

#[async_trait]
impl Projection for NodeListProjection {
    async fn handle_event(&mut self, _event: DomainEventEnum) -> Result<(), String> {
        // Handle graph domain events by extracting them from the enum
        // Note: This projection uses handle_graph_event for actual processing
        Ok(())
    }

    async fn clear(&mut self) -> Result<(), String> {
        self.nodes.clear();
        self.nodes_by_graph.clear();
        self.nodes_by_type.clear();
        self.checkpoint = None;
        Ok(())
    }

    async fn get_checkpoint(&self) -> Option<EventSequence> {
        self.checkpoint
    }

    async fn save_checkpoint(&mut self, sequence: EventSequence) -> Result<(), String> {
        self.checkpoint = Some(sequence);
        Ok(())
    }
}

#[async_trait]
impl super::GraphProjection for NodeListProjection {
    async fn handle_graph_event(&mut self, event: GraphDomainEvent) -> Result<(), String> {
        match event {
            GraphDomainEvent::NodeAdded(NodeAdded {
                graph_id,
                node_id,
                node_type,
                metadata,
                ..
            }) => {
                // Extract name from metadata if present
                let name = metadata
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let node_info = NodeInfo {
                    node_id,
                    graph_id,
                    node_type: node_type.clone(),
                    name,
                    metadata,
                };

                // Add to main index
                self.nodes.insert(node_id, node_info);

                // Add to graph index
                self.nodes_by_graph
                    .entry(graph_id)
                    .or_default()
                    .push(node_id);

                // Add to type index
                self.nodes_by_type
                    .entry(node_type)
                    .or_default()
                    .push(node_id);
            }

            GraphDomainEvent::NodeRemoved(NodeRemoved { graph_id, node_id }) => {
                // Remove from main index
                if let Some(node_info) = self.nodes.remove(&node_id) {
                    // Remove from graph index
                    if let Some(nodes) = self.nodes_by_graph.get_mut(&graph_id) {
                        nodes.retain(|id| id != &node_id);
                    }

                    // Remove from type index
                    if let Some(nodes) = self.nodes_by_type.get_mut(&node_info.node_type) {
                        nodes.retain(|id| id != &node_id);
                    }
                }
            }



            _ => {
                // Ignore other graph events
            }
        }

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::projections::GraphProjection;

    #[tokio::test]
    async fn test_node_list_projection() {
        let mut projection = NodeListProjection::new();
        let graph_id = GraphId::new();
        let node_id = NodeId::new();

        // Add a node
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), serde_json::Value::String("Test Node".to_string()));

        let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id,
            position: crate::value_objects::Position3D::default(),
            node_type: "TestType".to_string(),
            metadata,
        });

        projection.handle_graph_event(add_event).await.unwrap();

        // Verify node was added
        let node = projection.get_node(&node_id).unwrap();
        assert_eq!(node.node_type, "TestType");
        assert_eq!(node.name, Some("Test Node".to_string()));

        // Test search by name
        let results = projection.search_by_name("test");
        assert_eq!(results.len(), 1);

        // Test get by type
        let typed_nodes = projection.get_nodes_by_type("TestType");
        assert_eq!(typed_nodes.len(), 1);

        // Test get by graph
        let graph_nodes = projection.get_nodes_by_graph(&graph_id);
        assert_eq!(graph_nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_node_removal() {
        let mut projection = NodeListProjection::new();
        let graph_id = GraphId::new();
        let node_id = NodeId::new();

        // Add a node
        let add_event = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id,
            position: crate::value_objects::Position3D::default(),
            node_type: "TestType".to_string(),
            metadata: HashMap::new(),
        });

        projection.handle_graph_event(add_event).await.unwrap();
        assert_eq!(projection.total_nodes(), 1);

        // Remove the node
        let remove_event = GraphDomainEvent::NodeRemoved(NodeRemoved {
            graph_id,
            node_id,
        });

        projection.handle_graph_event(remove_event).await.unwrap();
        assert_eq!(projection.total_nodes(), 0);
        assert!(projection.get_node(&node_id).is_none());
    }
}
