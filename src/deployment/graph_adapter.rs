//! Adapter for working with Graph structures in deployment context

use crate::aggregate::business_graph::Graph;
use super::{DeploymentNodeType, DeploymentEdgeType};
use std::collections::HashMap;

/// Extension trait for Graph to provide deployment-specific functionality
pub trait DeploymentGraphExt {
    /// Get all nodes as a vector
    fn get_all_nodes(&self) -> Vec<DeploymentNode>;
    
    /// Get all edges as a vector
    fn get_all_edges(&self) -> Vec<DeploymentEdge>;
    
    /// Get a specific node by ID
    fn get_node(&self, node_id: &str) -> Option<DeploymentNode>;
    
    /// Get edges originating from a node
    fn get_edges_from(&self, node_id: &str) -> Vec<DeploymentEdge>;
    
    /// Get edges targeting a node
    fn get_edges_to(&self, node_id: &str) -> Vec<DeploymentEdge>;
}

/// Wrapper for a node with deployment data
#[derive(Debug, Clone)]
pub struct DeploymentNode {
    pub id: String,
    pub data: serde_json::Value,
}

/// Wrapper for an edge with deployment data
#[derive(Debug, Clone)]
pub struct DeploymentEdge {
    pub from: String,
    pub to: String,
    pub data: serde_json::Value,
}

impl DeploymentGraphExt for Graph {
    fn get_all_nodes(&self) -> Vec<DeploymentNode> {
        self.nodes()
            .iter()
            .filter_map(|(id, node)| {
                // Check if node has deployment metadata
                if let Some(deployment_data) = node.metadata.get("deployment") {
                    Some(DeploymentNode {
                        id: id.to_string(),
                        data: deployment_data.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn get_all_edges(&self) -> Vec<DeploymentEdge> {
        self.edges()
            .iter()
            .filter_map(|(_, edge)| {
                // Check if edge has deployment metadata
                if let Some(deployment_data) = edge.metadata.get("deployment") {
                    Some(DeploymentEdge {
                        from: edge.source_id.to_string(),
                        to: edge.target_id.to_string(),
                        data: deployment_data.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn get_node(&self, node_id: &str) -> Option<DeploymentNode> {
        // Try to parse the node_id string as a NodeId
        self.nodes()
            .iter()
            .find(|(id, _)| id.to_string() == node_id)
            .and_then(|(id, node)| {
                node.metadata.get("deployment").map(|data| DeploymentNode {
                    id: id.to_string(),
                    data: data.clone(),
                })
            })
    }
    
    fn get_edges_from(&self, node_id: &str) -> Vec<DeploymentEdge> {
        self.edges()
            .values()
            .filter(|edge| edge.source_id.to_string() == node_id)
            .filter_map(|edge| {
                edge.metadata.get("deployment").map(|data| DeploymentEdge {
                    from: edge.source_id.to_string(),
                    to: edge.target_id.to_string(),
                    data: data.clone(),
                })
            })
            .collect()
    }
    
    fn get_edges_to(&self, node_id: &str) -> Vec<DeploymentEdge> {
        self.edges()
            .values()
            .filter(|edge| edge.target_id.to_string() == node_id)
            .filter_map(|edge| {
                edge.metadata.get("deployment").map(|data| DeploymentEdge {
                    from: edge.source_id.to_string(),
                    to: edge.target_id.to_string(),
                    data: data.clone(),
                })
            })
            .collect()
    }
}

/// Helper to create deployment metadata for a node
pub fn create_deployment_node_metadata(node_type: DeploymentNodeType) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert("deployment".to_string(), serde_json::to_value(node_type).unwrap());
    metadata
}

/// Helper to create deployment metadata for an edge
pub fn create_deployment_edge_metadata(edge_type: DeploymentEdgeType) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert("deployment".to_string(), serde_json::to_value(edge_type).unwrap());
    metadata
}