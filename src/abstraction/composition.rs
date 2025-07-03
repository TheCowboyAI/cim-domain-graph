//! Graph composition module for merging multiple graphs
//!
//! This module provides functionality to compose multiple graphs into a single graph,
//! handling node/edge conflicts and preserving data integrity.

use super::{GraphType, GraphImplementation, NodeData, EdgeData, GraphOperationError};
use cim_domain::{GraphId, NodeId, EdgeId};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Errors that can occur during graph composition
#[derive(Debug, thiserror::Error)]
pub enum CompositionError {
    #[error("Node conflict: Node {0} exists in multiple graphs with different data")]
    NodeConflict(NodeId),
    
    #[error("Edge conflict: Edge {0} exists in multiple graphs with different data")]
    EdgeConflict(EdgeId),
    
    #[error("Invalid edge: Source node {0} or target node {1} not found in composed graph")]
    InvalidEdge(NodeId, NodeId),
    
    #[error("Graph operation failed: {0}")]
    GraphOperationFailed(#[from] GraphOperationError),
    
    #[error("Incompatible graph types: Cannot compose {0} with {1}")]
    IncompatibleGraphTypes(String, String),
}

/// Result type for composition operations
pub type CompositionResult<T> = Result<T, CompositionError>;

/// Strategy for resolving conflicts during composition
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictResolution {
    /// Keep the first occurrence, ignore subsequent ones
    KeepFirst,
    
    /// Replace with the last occurrence
    KeepLast,
    
    /// Merge data from all occurrences
    Merge,
    
    /// Fail on any conflict
    Fail,
}

/// Options for controlling composition behavior
#[derive(Debug, Clone)]
pub struct CompositionOptions {
    /// Strategy for resolving node conflicts
    pub node_conflict_resolution: ConflictResolution,
    
    /// Strategy for resolving edge conflicts
    pub edge_conflict_resolution: ConflictResolution,
    
    /// Whether to validate edges (ensure source/target nodes exist)
    pub validate_edges: bool,
    
    /// Whether to merge metadata when using Merge resolution
    pub merge_metadata: bool,
    
    /// Custom node ID mapping for avoiding conflicts
    pub node_id_mappings: HashMap<GraphId, HashMap<NodeId, NodeId>>,
    
    /// Custom edge ID mapping for avoiding conflicts
    pub edge_id_mappings: HashMap<GraphId, HashMap<EdgeId, EdgeId>>,
}

impl Default for CompositionOptions {
    fn default() -> Self {
        Self {
            node_conflict_resolution: ConflictResolution::Fail,
            edge_conflict_resolution: ConflictResolution::Fail,
            validate_edges: true,
            merge_metadata: true,
            node_id_mappings: HashMap::new(),
            edge_id_mappings: HashMap::new(),
        }
    }
}

/// Trait for graph composition operations
pub trait GraphComposer {
    /// Compose multiple graphs into a single graph
    fn compose(
        &self,
        graphs: &[&GraphType],
        target_type: &str,
        options: CompositionOptions,
    ) -> CompositionResult<GraphType>;
    
    /// Check if composition is valid for the given graphs
    fn validate_composition(&self, graphs: &[&GraphType]) -> CompositionResult<()>;
    
    /// Preview conflicts that would occur during composition
    fn preview_conflicts(&self, graphs: &[&GraphType]) -> (Vec<NodeId>, Vec<EdgeId>);
}

/// Default implementation of graph composer
pub struct DefaultGraphComposer;

impl DefaultGraphComposer {
    /// Create a new default graph composer
    pub fn new() -> Self {
        Self
    }
    
    /// Get the mapped node ID for a graph/node combination
    fn get_mapped_node_id(
        &self,
        graph_id: GraphId,
        node_id: NodeId,
        options: &CompositionOptions,
    ) -> NodeId {
        options.node_id_mappings
            .get(&graph_id)
            .and_then(|mapping| mapping.get(&node_id))
            .copied()
            .unwrap_or(node_id)
    }
    
    /// Get the mapped edge ID for a graph/edge combination
    fn get_mapped_edge_id(
        &self,
        graph_id: GraphId,
        edge_id: EdgeId,
        options: &CompositionOptions,
    ) -> EdgeId {
        options.edge_id_mappings
            .get(&graph_id)
            .and_then(|mapping| mapping.get(&edge_id))
            .copied()
            .unwrap_or(edge_id)
    }
    
    /// Merge two metadata maps
    fn merge_metadata(
        &self,
        mut base: HashMap<String, Value>,
        other: HashMap<String, Value>,
    ) -> HashMap<String, Value> {
        for (key, value) in other {
            if let Some(base_value) = base.get_mut(&key) {
                // If both have the same key, try to merge intelligently
                match (base_value, value) {
                    (Value::Array(base_arr), Value::Array(other_arr)) => {
                        // Merge arrays by concatenating unique values
                        for item in other_arr {
                            if !base_arr.contains(&item) {
                                base_arr.push(item);
                            }
                        }
                    }
                    (Value::Object(base_obj), Value::Object(other_obj)) => {
                        // Recursively merge objects
                        for (k, v) in other_obj {
                            base_obj.insert(k, v);
                        }
                    }
                    (base_val, other_val) => {
                        // For other types, keep the other value
                        *base_val = other_val;
                    }
                }
            } else {
                // Key doesn't exist in base, add it
                base.insert(key, value);
            }
        }
        base
    }
    
    /// Resolve node conflict based on strategy
    fn resolve_node_conflict(
        &self,
        existing: &NodeData,
        new: &NodeData,
        strategy: ConflictResolution,
        merge_metadata: bool,
    ) -> CompositionResult<NodeData> {
        match strategy {
            ConflictResolution::KeepFirst => Ok(existing.clone()),
            ConflictResolution::KeepLast => Ok(new.clone()),
            ConflictResolution::Merge => {
                let mut merged = existing.clone();
                
                // Merge positions by averaging
                merged.position.x = (existing.position.x + new.position.x) / 2.0;
                merged.position.y = (existing.position.y + new.position.y) / 2.0;
                merged.position.z = (existing.position.z + new.position.z) / 2.0;
                
                // Merge metadata if requested
                if merge_metadata {
                    merged.metadata = self.merge_metadata(
                        existing.metadata.clone(),
                        new.metadata.clone(),
                    );
                }
                
                // Keep the node type from the first occurrence
                // (or we could make this configurable)
                
                Ok(merged)
            }
            ConflictResolution::Fail => {
                // Check if they're actually different
                if existing.node_type != new.node_type
                    || existing.position != new.position
                    || existing.metadata != new.metadata
                {
                    Err(CompositionError::NodeConflict(NodeId::new()))
                } else {
                    // They're the same, no conflict
                    Ok(existing.clone())
                }
            }
        }
    }
    
    /// Resolve edge conflict based on strategy
    fn resolve_edge_conflict(
        &self,
        existing: &EdgeData,
        new: &EdgeData,
        strategy: ConflictResolution,
        merge_metadata: bool,
    ) -> CompositionResult<EdgeData> {
        match strategy {
            ConflictResolution::KeepFirst => Ok(existing.clone()),
            ConflictResolution::KeepLast => Ok(new.clone()),
            ConflictResolution::Merge => {
                let mut merged = existing.clone();
                
                // Merge metadata if requested
                if merge_metadata {
                    merged.metadata = self.merge_metadata(
                        existing.metadata.clone(),
                        new.metadata.clone(),
                    );
                }
                
                // Keep the edge type from the first occurrence
                // (or we could make this configurable)
                
                Ok(merged)
            }
            ConflictResolution::Fail => {
                // Check if they're actually different
                if existing.edge_type != new.edge_type
                    || existing.metadata != new.metadata
                {
                    Err(CompositionError::EdgeConflict(EdgeId::new()))
                } else {
                    // They're the same, no conflict
                    Ok(existing.clone())
                }
            }
        }
    }
}

impl GraphComposer for DefaultGraphComposer {
    fn compose(
        &self,
        graphs: &[&GraphType],
        target_type: &str,
        options: CompositionOptions,
    ) -> CompositionResult<GraphType> {
        if graphs.is_empty() {
            return Err(CompositionError::GraphOperationFailed(
                GraphOperationError::InvalidOperation("Cannot compose zero graphs".to_string())
            ));
        }
        
        // Validate composition
        self.validate_composition(graphs)?;
        
        // Create target graph
        let target_id = GraphId::new();
        let mut target = match target_type {
            "context" => GraphType::new_context(target_id, "Composed Graph"),
            "concept" => GraphType::new_concept(target_id, "Composed Graph"),
            "workflow" => GraphType::new_workflow(target_id, "Composed Graph"),
            "ipld" => GraphType::new_ipld(target_id),
            _ => return Err(CompositionError::GraphOperationFailed(
                GraphOperationError::InvalidOperation(format!("Unknown graph type: {target_type}"))
            )),
        };
        
        // Track nodes and edges we've already added
        let mut node_map: HashMap<NodeId, NodeData> = HashMap::new();
        let mut edge_map: HashMap<EdgeId, (EdgeData, NodeId, NodeId)> = HashMap::new();
        
        // First pass: collect all nodes
        for graph in graphs {
            let graph_id = graph.graph_id();
            
            for (node_id, node_data) in graph.list_nodes() {
                let mapped_id = self.get_mapped_node_id(graph_id, node_id, &options);
                
                if let Some(existing) = node_map.get(&mapped_id) {
                    // Conflict detected
                    let resolved = self.resolve_node_conflict(
                        existing,
                        &node_data,
                        options.node_conflict_resolution,
                        options.merge_metadata,
                    )?;
                    node_map.insert(mapped_id, resolved);
                } else {
                    // No conflict, add the node
                    node_map.insert(mapped_id, node_data);
                }
            }
        }
        
        // Add all nodes to the target graph
        for (node_id, node_data) in &node_map {
            target.add_node(*node_id, node_data.clone())?;
        }
        
        // Second pass: collect all edges
        for graph in graphs {
            let graph_id = graph.graph_id();
            
            for (edge_id, edge_data, source, target_id) in graph.list_edges() {
                let mapped_edge_id = self.get_mapped_edge_id(graph_id, edge_id, &options);
                let mapped_source = self.get_mapped_node_id(graph_id, source, &options);
                let mapped_target = self.get_mapped_node_id(graph_id, target_id, &options);
                
                if let Some((existing, _, _)) = edge_map.get(&mapped_edge_id) {
                    // Conflict detected
                    let resolved = self.resolve_edge_conflict(
                        existing,
                        &edge_data,
                        options.edge_conflict_resolution,
                        options.merge_metadata,
                    )?;
                    edge_map.insert(mapped_edge_id, (resolved, mapped_source, mapped_target));
                } else {
                    // No conflict, add the edge
                    edge_map.insert(mapped_edge_id, (edge_data, mapped_source, mapped_target));
                }
            }
        }
        
        // Validate edges if requested
        if options.validate_edges {
            for (_edge_id, (_, source, target_id)) in &edge_map {
                if !node_map.contains_key(source) || !node_map.contains_key(target_id) {
                    return Err(CompositionError::InvalidEdge(*source, *target_id));
                }
            }
        }
        
        // Add all edges to the target graph
        for (edge_id, (edge_data, source, target_id)) in edge_map {
            target.add_edge(edge_id, source, target_id, edge_data)?;
        }
        
        Ok(target)
    }
    
    fn validate_composition(&self, graphs: &[&GraphType]) -> CompositionResult<()> {
        if graphs.is_empty() {
            return Err(CompositionError::GraphOperationFailed(
                GraphOperationError::InvalidOperation("Cannot compose zero graphs".to_string())
            ));
        }
        
        // For now, we allow composing any graph types
        // In the future, we might want to restrict certain combinations
        
        Ok(())
    }
    
    fn preview_conflicts(&self, graphs: &[&GraphType]) -> (Vec<NodeId>, Vec<EdgeId>) {
        let mut node_conflicts = Vec::new();
        let mut edge_conflicts = Vec::new();
        
        let mut seen_nodes = HashSet::new();
        let mut seen_edges = HashSet::new();
        
        for graph in graphs {
            for (node_id, _) in graph.list_nodes() {
                if !seen_nodes.insert(node_id) {
                    node_conflicts.push(node_id);
                }
            }
            
            for (edge_id, _, _, _) in graph.list_edges() {
                if !seen_edges.insert(edge_id) {
                    edge_conflicts.push(edge_id);
                }
            }
        }
        
        (node_conflicts, edge_conflicts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_composition_options_default() {
        let options = CompositionOptions::default();
        assert_eq!(options.node_conflict_resolution, ConflictResolution::Fail);
        assert_eq!(options.edge_conflict_resolution, ConflictResolution::Fail);
        assert!(options.validate_edges);
        assert!(options.merge_metadata);
    }
    
    #[test]
    fn test_metadata_merging() {
        let composer = DefaultGraphComposer::new();
        
        let mut base = HashMap::new();
        base.insert("key1".to_string(), serde_json::json!("value1"));
        base.insert("array".to_string(), serde_json::json!(["a", "b"]));
        
        let mut other = HashMap::new();
        other.insert("key2".to_string(), serde_json::json!("value2"));
        other.insert("array".to_string(), serde_json::json!(["c", "d"]));
        
        let merged = composer.merge_metadata(base, other);
        
        assert_eq!(merged.get("key1").unwrap(), &serde_json::json!("value1"));
        assert_eq!(merged.get("key2").unwrap(), &serde_json::json!("value2"));
        
        // Arrays should be merged
        let array = merged.get("array").unwrap().as_array().unwrap();
        assert_eq!(array.len(), 4);
        assert!(array.contains(&serde_json::json!("a")));
        assert!(array.contains(&serde_json::json!("c")));
    }
} 