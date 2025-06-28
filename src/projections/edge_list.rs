//! Edge list projection
//!
//! Provides a searchable list of all edges across graphs.

use crate::{
    domain_events::GraphDomainEvent,
    events::{EdgeAdded, EdgeRemoved},
    EdgeId, GraphId, NodeId,
};
use async_trait::async_trait;
use cim_domain::projections::{EventSequence, Projection};
use cim_domain::DomainEventEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about an edge for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    /// Unique identifier of the edge
    pub edge_id: EdgeId,
    /// ID of the graph this edge belongs to
    pub graph_id: GraphId,
    /// Source node ID
    pub source_id: NodeId,
    /// Target node ID
    pub target_id: NodeId,
    /// Type/relationship of the edge
    pub edge_type: String,
    /// Additional metadata about the edge
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Projection that maintains a searchable list of edges
#[derive(Debug, Clone)]
pub struct EdgeListProjection {
    edges: HashMap<EdgeId, EdgeInfo>,
    edges_by_graph: HashMap<GraphId, Vec<EdgeId>>,
    edges_by_type: HashMap<String, Vec<EdgeId>>,
    edges_by_node: HashMap<NodeId, Vec<EdgeId>>,
    incoming_edges: HashMap<NodeId, Vec<EdgeId>>,
    outgoing_edges: HashMap<NodeId, Vec<EdgeId>>,
    checkpoint: Option<EventSequence>,
}

impl Default for EdgeListProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeListProjection {
    /// Create a new edge list projection
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            edges_by_graph: HashMap::new(),
            edges_by_type: HashMap::new(),
            edges_by_node: HashMap::new(),
            incoming_edges: HashMap::new(),
            outgoing_edges: HashMap::new(),
            checkpoint: None,
        }
    }

    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: &EdgeId) -> Option<&EdgeInfo> {
        self.edges.get(edge_id)
    }

    /// Get all edges
    pub fn get_all_edges(&self) -> Vec<&EdgeInfo> {
        self.edges.values().collect()
    }

    /// Get edges by graph ID
    pub fn get_edges_by_graph(&self, graph_id: &GraphId) -> Vec<&EdgeInfo> {
        self.edges_by_graph
            .get(graph_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get edges by type
    pub fn get_edges_by_type(&self, edge_type: &str) -> Vec<&EdgeInfo> {
        self.edges_by_type
            .get(edge_type)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all edges connected to a node (incoming and outgoing)
    pub fn get_edges_for_node(&self, node_id: &NodeId) -> Vec<&EdgeInfo> {
        self.edges_by_node
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get incoming edges for a node
    pub fn get_incoming_edges(&self, node_id: &NodeId) -> Vec<&EdgeInfo> {
        self.incoming_edges
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get outgoing edges for a node
    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<&EdgeInfo> {
        self.outgoing_edges
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get total number of edges
    pub fn total_edges(&self) -> usize {
        self.edges.len()
    }

    /// Get edge count for a specific graph
    pub fn get_edge_count_for_graph(&self, graph_id: &GraphId) -> usize {
        self.edges_by_graph
            .get(graph_id)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    /// Get edge count by type
    pub fn count_by_type(&self) -> HashMap<String, usize> {
        self.edges_by_type
            .iter()
            .map(|(edge_type, ids)| (edge_type.clone(), ids.len()))
            .collect()
    }

    /// Build adjacency list for a graph
    pub fn get_adjacency_list(&self, graph_id: &GraphId) -> HashMap<NodeId, Vec<NodeId>> {
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        
        if let Some(edge_ids) = self.edges_by_graph.get(graph_id) {
            for edge_id in edge_ids {
                if let Some(edge) = self.edges.get(edge_id) {
                    adjacency
                        .entry(edge.source_id)
                        .or_default()
                        .push(edge.target_id);
                }
            }
        }
        
        adjacency
    }
}

#[async_trait]
impl Projection for EdgeListProjection {
    async fn handle_event(&mut self, _event: DomainEventEnum) -> Result<(), String> {
        // Handle graph domain events by extracting them from the enum
        // Note: This projection uses handle_graph_event for actual processing
        Ok(())
    }

    async fn clear(&mut self) -> Result<(), String> {
        self.edges.clear();
        self.edges_by_graph.clear();
        self.edges_by_type.clear();
        self.edges_by_node.clear();
        self.incoming_edges.clear();
        self.outgoing_edges.clear();
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
impl super::GraphProjection for EdgeListProjection {
    async fn handle_graph_event(&mut self, event: GraphDomainEvent) -> Result<(), String> {
        match event {
            GraphDomainEvent::EdgeAdded(EdgeAdded {
                graph_id,
                edge_id,
                source,
                target,
                relationship: _,
                edge_type,
                metadata,
            }) => {
                let edge_info = EdgeInfo {
                    edge_id,
                    graph_id,
                    source_id: source,
                    target_id: target,
                    edge_type: edge_type.clone(),
                    metadata,
                };

                // Add to main index
                self.edges.insert(edge_id, edge_info);

                // Add to graph index
                self.edges_by_graph
                    .entry(graph_id)
                    .or_default()
                    .push(edge_id);

                // Add to type index
                self.edges_by_type
                    .entry(edge_type)
                    .or_default()
                    .push(edge_id);

                // Add to node indices
                self.edges_by_node
                    .entry(source)
                    .or_default()
                    .push(edge_id);
                self.edges_by_node
                    .entry(target)
                    .or_default()
                    .push(edge_id);

                // Add to directional indices
                self.outgoing_edges
                    .entry(source)
                    .or_default()
                    .push(edge_id);
                self.incoming_edges
                    .entry(target)
                    .or_default()
                    .push(edge_id);
            }

            GraphDomainEvent::EdgeRemoved(EdgeRemoved { graph_id, edge_id }) => {
                // Remove from main index
                if let Some(edge_info) = self.edges.remove(&edge_id) {
                    // Remove from graph index
                    if let Some(edges) = self.edges_by_graph.get_mut(&graph_id) {
                        edges.retain(|id| id != &edge_id);
                    }

                    // Remove from type index
                    if let Some(edges) = self.edges_by_type.get_mut(&edge_info.edge_type) {
                        edges.retain(|id| id != &edge_id);
                    }

                    // Remove from node indices
                    if let Some(edges) = self.edges_by_node.get_mut(&edge_info.source_id) {
                        edges.retain(|id| id != &edge_id);
                    }
                    if let Some(edges) = self.edges_by_node.get_mut(&edge_info.target_id) {
                        edges.retain(|id| id != &edge_id);
                    }

                    // Remove from directional indices
                    if let Some(edges) = self.outgoing_edges.get_mut(&edge_info.source_id) {
                        edges.retain(|id| id != &edge_id);
                    }
                    if let Some(edges) = self.incoming_edges.get_mut(&edge_info.target_id) {
                        edges.retain(|id| id != &edge_id);
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
    use crate::components::EdgeRelationship;

    #[tokio::test]
    async fn test_edge_list_projection() {
        let mut projection = EdgeListProjection::new();
        let graph_id = GraphId::new();
        let edge_id = EdgeId::new();
        let source_id = NodeId::new();
        let target_id = NodeId::new();

        // Add an edge
        let add_event = GraphDomainEvent::EdgeAdded(EdgeAdded {
            graph_id,
            edge_id,
            source: source_id,
            target: target_id,
            relationship: EdgeRelationship::Dependency {
                dependency_type: "test".to_string(),
                strength: 1.0,
            },
            edge_type: "dependency".to_string(),
            metadata: HashMap::new(),
        });

        projection.handle_graph_event(add_event).await.unwrap();

        // Verify edge was added
        let edge = projection.get_edge(&edge_id).unwrap();
        assert_eq!(edge.edge_type, "dependency");
        assert_eq!(edge.source_id, source_id);
        assert_eq!(edge.target_id, target_id);

        // Test get by type
        let typed_edges = projection.get_edges_by_type("dependency");
        assert_eq!(typed_edges.len(), 1);

        // Test get by graph
        let graph_edges = projection.get_edges_by_graph(&graph_id);
        assert_eq!(graph_edges.len(), 1);

        // Test directional queries
        let outgoing = projection.get_outgoing_edges(&source_id);
        assert_eq!(outgoing.len(), 1);

        let incoming = projection.get_incoming_edges(&target_id);
        assert_eq!(incoming.len(), 1);
    }

    #[tokio::test]
    async fn test_edge_removal() {
        let mut projection = EdgeListProjection::new();
        let graph_id = GraphId::new();
        let edge_id = EdgeId::new();
        let source_id = NodeId::new();
        let target_id = NodeId::new();

        // Add an edge
        let add_event = GraphDomainEvent::EdgeAdded(EdgeAdded {
            graph_id,
            edge_id,
            source: source_id,
            target: target_id,
            relationship: EdgeRelationship::Similarity { score: 0.8 },
            edge_type: "similarity".to_string(),
            metadata: HashMap::new(),
        });

        projection.handle_graph_event(add_event).await.unwrap();
        assert_eq!(projection.total_edges(), 1);

        // Remove the edge
        let remove_event = GraphDomainEvent::EdgeRemoved(EdgeRemoved { graph_id, edge_id });

        projection.handle_graph_event(remove_event).await.unwrap();
        assert_eq!(projection.total_edges(), 0);
        assert!(projection.get_edge(&edge_id).is_none());
    }

    #[tokio::test]
    async fn test_adjacency_list() {
        let mut projection = EdgeListProjection::new();
        let graph_id = GraphId::new();
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        // Add edges: node1 -> node2, node1 -> node3
        let edge1 = GraphDomainEvent::EdgeAdded(EdgeAdded {
            graph_id,
            edge_id: EdgeId::new(),
            source: node1,
            target: node2,
            relationship: EdgeRelationship::Dependency {
                dependency_type: "test".to_string(),
                strength: 1.0,
            },
            edge_type: "dependency".to_string(),
            metadata: HashMap::new(),
        });

        let edge2 = GraphDomainEvent::EdgeAdded(EdgeAdded {
            graph_id,
            edge_id: EdgeId::new(),
            source: node1,
            target: node3,
            relationship: EdgeRelationship::Dependency {
                dependency_type: "test".to_string(),
                strength: 1.0,
            },
            edge_type: "dependency".to_string(),
            metadata: HashMap::new(),
        });

        projection.handle_graph_event(edge1).await.unwrap();
        projection.handle_graph_event(edge2).await.unwrap();

        // Get adjacency list
        let adjacency = projection.get_adjacency_list(&graph_id);
        assert_eq!(adjacency.get(&node1).unwrap().len(), 2);
        assert!(adjacency.get(&node1).unwrap().contains(&node2));
        assert!(adjacency.get(&node1).unwrap().contains(&node3));
    }
} 