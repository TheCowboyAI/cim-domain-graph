//! Graph transformation module for converting between different graph types
//!
//! This module provides functionality to transform graphs from one type to another
//! while preserving data integrity and handling type-specific conversions.

use super::{GraphType, GraphImplementation, GraphMetadata, NodeData, EdgeData, GraphOperationError};
use cim_domain::GraphId;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Errors that can occur during graph transformation
#[derive(Debug, thiserror::Error)]
pub enum TransformationError {
    #[error("Transformation not supported from {0} to {1}")]
    UnsupportedTransformation(String, String),
    
    #[error("Data loss would occur: {0}")]
    DataLossWarning(String),
    
    #[error("Invalid metadata for target type: {0}")]
    InvalidMetadata(String),
    
    #[error("Node type '{0}' cannot be transformed to target graph type")]
    IncompatibleNodeType(String),
    
    #[error("Edge type '{0}' cannot be transformed to target graph type")]
    IncompatibleEdgeType(String),
    
    #[error("Graph operation failed: {0}")]
    GraphOperationFailed(#[from] GraphOperationError),
}

/// Result type for transformation operations
pub type TransformationResult<T> = Result<T, TransformationError>;

/// Options for controlling transformation behavior
#[derive(Debug, Clone, Default)]
pub struct TransformationOptions {
    /// Whether to allow data loss during transformation
    pub allow_data_loss: bool,
    
    /// Whether to preserve original metadata
    pub preserve_metadata: bool,
    
    /// Custom node type mappings
    pub node_type_mappings: HashMap<String, String>,
    
    /// Custom edge type mappings
    pub edge_type_mappings: HashMap<String, String>,
    
    /// Additional metadata to add during transformation
    pub additional_metadata: HashMap<String, Value>,
}

/// Trait for graph transformations
pub trait GraphTransformer {
    /// Transform a graph from one type to another
    fn transform(
        &self,
        source: &GraphType,
        target_type: &str,
        options: TransformationOptions,
    ) -> TransformationResult<GraphType>;
    
    /// Check if a transformation is supported
    fn is_transformation_supported(&self, from: &str, to: &str) -> bool;
    
    /// Preview what data might be lost in a transformation
    fn preview_data_loss(&self, source: &GraphType, target_type: &str) -> Vec<String>;
}

/// Default implementation of graph transformer
pub struct DefaultGraphTransformer;

impl Default for DefaultGraphTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultGraphTransformer {
    /// Create a new default graph transformer
    pub fn new() -> Self {
        Self
    }
    
    /// Get the type name of a GraphType
    fn get_type_name(graph_type: &GraphType) -> &'static str {
        match graph_type {
            GraphType::Context(_) => "context",
            GraphType::Concept(_) => "concept",
            GraphType::Workflow(_) => "workflow",
            GraphType::Ipld(_) => "ipld",
        }
    }
    
    /// Transform node data for the target graph type
    fn transform_node_data(
        &self,
        node_data: &NodeData,
        target_type: &str,
        options: &TransformationOptions,
    ) -> TransformationResult<NodeData> {
        let mut transformed = node_data.clone();
        
        // Apply node type mappings if provided
        if let Some(mapped_type) = options.node_type_mappings.get(&node_data.node_type) {
            transformed.node_type = mapped_type.clone();
        }
        
        // Add type-specific metadata transformations
        match target_type {
            "workflow" => {
                // Ensure workflow nodes have required metadata
                if !transformed.metadata.contains_key("step_type") {
                    transformed.metadata.insert(
                        "step_type".to_string(),
                        json!("process"),
                    );
                }
            }
            "concept" => {
                // Add conceptual space coordinates if not present
                if !transformed.metadata.contains_key("conceptual_coordinates") {
                    transformed.metadata.insert(
                        "conceptual_coordinates".to_string(),
                        json!([0.0, 0.0, 0.0]),
                    );
                }
            }
            "context" => {
                // Add context information if not present
                if !transformed.metadata.contains_key("context_type") {
                    transformed.metadata.insert(
                        "context_type".to_string(),
                        json!("general"),
                    );
                }
            }
            "ipld" => {
                // Ensure IPLD nodes have CID information
                if !transformed.metadata.contains_key("cid") {
                    transformed.metadata.insert(
                        "cid".to_string(),
                        json!(null),
                    );
                }
            }
            _ => {}
        }
        
        // Add any additional metadata from options
        for (key, value) in &options.additional_metadata {
            transformed.metadata.insert(key.clone(), value.clone());
        }
        
        Ok(transformed)
    }
    
    /// Transform edge data for the target graph type
    fn transform_edge_data(
        &self,
        edge_data: &EdgeData,
        target_type: &str,
        options: &TransformationOptions,
    ) -> TransformationResult<EdgeData> {
        let mut transformed = edge_data.clone();
        
        // Apply edge type mappings if provided
        if let Some(mapped_type) = options.edge_type_mappings.get(&edge_data.edge_type) {
            transformed.edge_type = mapped_type.clone();
        }
        
        // Add type-specific edge transformations
        match target_type {
            "workflow" => {
                // Ensure workflow edges have flow type
                if !transformed.metadata.contains_key("flow_type") {
                    transformed.metadata.insert(
                        "flow_type".to_string(),
                        json!("sequence"),
                    );
                }
            }
            "concept" => {
                // Add semantic relationship strength if not present
                if !transformed.metadata.contains_key("semantic_strength") {
                    transformed.metadata.insert(
                        "semantic_strength".to_string(),
                        json!(0.5),
                    );
                }
            }
            "context" => {
                // Add context relationship type
                if !transformed.metadata.contains_key("relationship_context") {
                    transformed.metadata.insert(
                        "relationship_context".to_string(),
                        json!("default"),
                    );
                }
            }
            "ipld" => {
                // Add IPLD link information
                if !transformed.metadata.contains_key("link_type") {
                    transformed.metadata.insert(
                        "link_type".to_string(),
                        json!("reference"),
                    );
                }
            }
            _ => {}
        }
        
        Ok(transformed)
    }
    
    /// Create a new graph of the specified type
    fn create_target_graph(
        &self,
        graph_id: GraphId,
        target_type: &str,
        source_metadata: &GraphMetadata,
    ) -> TransformationResult<GraphType> {
        let graph = match target_type {
            "context" => GraphType::new_context(graph_id, &source_metadata.name),
            "concept" => GraphType::new_concept(graph_id, &source_metadata.name),
            "workflow" => GraphType::new_workflow(graph_id, &source_metadata.name),
            "ipld" => GraphType::new_ipld(graph_id),
            _ => return Err(TransformationError::UnsupportedTransformation(
                "unknown".to_string(),
                target_type.to_string(),
            )),
        };
        
        Ok(graph)
    }
}

impl GraphTransformer for DefaultGraphTransformer {
    fn transform(
        &self,
        source: &GraphType,
        target_type: &str,
        options: TransformationOptions,
    ) -> TransformationResult<GraphType> {
        let source_type = Self::get_type_name(source);
        
        // Check if transformation is supported
        if !self.is_transformation_supported(source_type, target_type) {
            return Err(TransformationError::UnsupportedTransformation(
                source_type.to_string(),
                target_type.to_string(),
            ));
        }
        
        // Check for potential data loss
        if !options.allow_data_loss {
            let data_loss = self.preview_data_loss(source, target_type);
            if !data_loss.is_empty() {
                return Err(TransformationError::DataLossWarning(
                    data_loss.join(", ")
                ));
            }
        }
        
        // Get source metadata
        let source_metadata = source.get_metadata();
        
        // Create target graph
        let mut target = self.create_target_graph(
            source.graph_id(),
            target_type,
            &source_metadata,
        )?;
        
        // Update metadata if needed
        if options.preserve_metadata {
            let mut target_metadata = source_metadata.clone();
            target_metadata.properties.insert(
                "transformed_from".to_string(),
                json!(source_type),
            );
            target.update_metadata(target_metadata)?;
        }
        
        // Transform and copy nodes
        for (node_id, node_data) in source.list_nodes() {
            let transformed_node = self.transform_node_data(
                &node_data,
                target_type,
                &options,
            )?;
            target.add_node(node_id, transformed_node)?;
        }
        
        // Transform and copy edges
        for (edge_id, edge_data, source_id, target_id) in source.list_edges() {
            let transformed_edge = self.transform_edge_data(
                &edge_data,
                target_type,
                &options,
            )?;
            target.add_edge(edge_id, source_id, target_id, transformed_edge)?;
        }
        
        Ok(target)
    }
    
    fn is_transformation_supported(&self, from: &str, to: &str) -> bool {
        // All transformations are supported by default
        // This can be customized based on specific requirements
        matches!(
            (from, to),
            ("context", "concept") | ("context", "workflow") | ("context", "ipld") |
            ("concept", "context") | ("concept", "workflow") | ("concept", "ipld") |
            ("workflow", "context") | ("workflow", "concept") | ("workflow", "ipld") |
            ("ipld", "context") | ("ipld", "concept") | ("ipld", "workflow")
        )
    }
    
    fn preview_data_loss(&self, source: &GraphType, target_type: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        let source_type = Self::get_type_name(source);
        
        // Check for type-specific data that might be lost
        match (source_type, target_type) {
            ("workflow", "concept") => {
                // Check for workflow-specific metadata
                for (_, node_data) in source.list_nodes() {
                    if node_data.metadata.contains_key("execution_time") {
                        warnings.push("Workflow execution time metadata will be lost".to_string());
                        break;
                    }
                }
            }
            ("concept", "workflow") => {
                // Check for concept-specific metadata
                for (_, node_data) in source.list_nodes() {
                    if node_data.metadata.contains_key("semantic_embedding") {
                        warnings.push("Conceptual embeddings will be lost".to_string());
                        break;
                    }
                }
            }
            ("ipld", _) => {
                // Check for IPLD-specific data
                for (_, node_data) in source.list_nodes() {
                    if node_data.metadata.contains_key("ipld_codec") {
                        warnings.push("IPLD codec information will be lost".to_string());
                        break;
                    }
                }
            }
            _ => {}
        }
        
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transformation_support() {
        let transformer = DefaultGraphTransformer::new();
        
        // Test all supported transformations
        assert!(transformer.is_transformation_supported("context", "concept"));
        assert!(transformer.is_transformation_supported("workflow", "ipld"));
        assert!(transformer.is_transformation_supported("concept", "context"));
        
        // Test unsupported transformation
        assert!(!transformer.is_transformation_supported("unknown", "context"));
    }
    
    #[test]
    fn test_node_data_transformation() {
        let transformer = DefaultGraphTransformer::new();
        let options = TransformationOptions::default();
        
        let node_data = NodeData {
            node_type: "test_node".to_string(),
            position: super::super::Position3D::default(),
            metadata: HashMap::new(),
        };
        
        // Transform to workflow type
        let result = transformer.transform_node_data(&node_data, "workflow", &options).unwrap();
        assert!(result.metadata.contains_key("step_type"));
        
        // Transform to concept type
        let result = transformer.transform_node_data(&node_data, "concept", &options).unwrap();
        assert!(result.metadata.contains_key("conceptual_coordinates"));
    }
} 