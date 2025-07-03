//! Adapter for ConceptGraph implementation

use crate::abstraction::{
    GraphImplementation, GraphMetadata, GraphOperationError, GraphResult,
    NodeData, EdgeData, Position3D,
};
use cim_domain::{NodeId, EdgeId, GraphId};
use cim_conceptgraph::{ConceptGraph, SemanticRelationship, ConceptId, ConceptualPoint};
use cim_contextgraph::{NodeId as ContextNodeId, EdgeId as ContextEdgeId};
use std::collections::HashMap;

/// Adapter that wraps ConceptGraph to implement GraphImplementation
#[derive(Clone)]
pub struct ConceptGraphAdapter {
    graph: std::sync::Arc<std::sync::Mutex<ConceptGraph>>,
    graph_id: GraphId,
    // Map domain NodeId to ConceptGraph's internal NodeId
    node_id_map: HashMap<NodeId, ContextNodeId>,
    reverse_node_map: HashMap<ContextNodeId, NodeId>,
    // Map domain EdgeId to ConceptGraph's internal EdgeId
    edge_id_map: HashMap<EdgeId, ContextEdgeId>,
    reverse_edge_map: HashMap<ContextEdgeId, EdgeId>,
    // Store original metadata to preserve it
    node_metadata: HashMap<NodeId, HashMap<String, serde_json::Value>>,
    edge_metadata: HashMap<EdgeId, HashMap<String, serde_json::Value>>,
    // Store original node types
    node_types: HashMap<NodeId, String>,
}

impl ConceptGraphAdapter {
    /// Create a new adapter
    pub fn new(graph_id: GraphId, name: &str) -> Self {
        Self {
            graph: std::sync::Arc::new(std::sync::Mutex::new(ConceptGraph::new(name))),
            graph_id,
            node_id_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            edge_id_map: HashMap::new(),
            reverse_edge_map: HashMap::new(),
            node_metadata: HashMap::new(),
            edge_metadata: HashMap::new(),
            node_types: HashMap::new(),
        }
    }
}

impl GraphImplementation for ConceptGraphAdapter {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
    
    fn add_node(&mut self, node_id: NodeId, data: NodeData) -> GraphResult<()> {
        // Store original metadata and type
        self.node_metadata.insert(node_id, data.metadata.clone());
        self.node_types.insert(node_id, data.node_type.clone());
        
        // Create a ConceptNode from NodeData
        let concept_id = ConceptId::new();
        let position = ConceptualPoint {
            coordinates: vec![
                data.position.x as f32,
                data.position.y as f32,
                data.position.z as f32,
            ],
        };
        
        let label = data.metadata.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(&data.node_type)
            .to_string();
        
        let ctx_node_id = self.graph.lock().unwrap().add_concept(concept_id, position, label);
        
        // Store mapping
        self.node_id_map.insert(node_id, ctx_node_id);
        self.reverse_node_map.insert(ctx_node_id, node_id);
        
        Ok(())
    }
    
    fn add_edge(&mut self, edge_id: EdgeId, source: NodeId, target: NodeId, data: EdgeData) -> GraphResult<()> {
        // Store original metadata
        self.edge_metadata.insert(edge_id, data.metadata.clone());
        
        let source_ctx = self.node_id_map.get(&source)
            .ok_or(GraphOperationError::NodeNotFound(source))?;
        let target_ctx = self.node_id_map.get(&target)
            .ok_or(GraphOperationError::NodeNotFound(target))?;
        
        // Convert edge type to semantic relationship
        let relationship = match data.edge_type.as_str() {
            "similarity" => SemanticRelationship::Similarity,
            "hierarchy" => SemanticRelationship::Hierarchy,
            "meronymy" => SemanticRelationship::Meronymy,
            "causality" => SemanticRelationship::Causality,
            other => SemanticRelationship::Custom(other.to_string()),
        };
        
        let strength = data.metadata.get("strength")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5) as f32;
        
        let ctx_edge_id = self.graph.lock().unwrap().connect_concepts(*source_ctx, *target_ctx, relationship, strength)
            .map_err(|e| GraphOperationError::EdgeCreationFailed(e.to_string()))?;
        
        // Store mapping
        self.edge_id_map.insert(edge_id, ctx_edge_id);
        self.reverse_edge_map.insert(ctx_edge_id, edge_id);
        
        Ok(())
    }
    
    fn get_node(&self, node_id: NodeId) -> GraphResult<NodeData> {
        let ctx_id = self.node_id_map.get(&node_id)
            .ok_or(GraphOperationError::NodeNotFound(node_id))?;
        
        let graph = self.graph.lock().unwrap();
        let node = graph.get_node(*ctx_id)
            .ok_or(GraphOperationError::NodeNotFound(node_id))?
            .clone();
        
        let position = Position3D {
            x: node.position.coordinates.get(0).copied().unwrap_or(0.0) as f64,
            y: node.position.coordinates.get(1).copied().unwrap_or(0.0) as f64,
            z: node.position.coordinates.get(2).copied().unwrap_or(0.0) as f64,
        };
        
        // Start with original metadata if available
        let mut metadata = self.node_metadata.get(&node_id)
            .cloned()
            .unwrap_or_else(|| node.metadata.clone());
        
        // Add/override concept-specific fields
        metadata.insert("label".to_string(), serde_json::Value::String(node.label.clone()));
        metadata.insert("concept_id".to_string(), serde_json::Value::String(format!("{:?}", node.concept_id)));
        
        // Merge any additional metadata from the concept node
        for (key, value) in node.metadata {
            if !metadata.contains_key(&key) {
                metadata.insert(key, value);
            }
        }
        
        // Get original node type or default to "concept"
        let node_type = self.node_types.get(&node_id)
            .cloned()
            .unwrap_or_else(|| "concept".to_string());
        
        Ok(NodeData {
            node_type,
            position,
            metadata,
        })
    }
    
    fn get_edge(&self, edge_id: EdgeId) -> GraphResult<(EdgeData, NodeId, NodeId)> {
        let ctx_id = self.edge_id_map.get(&edge_id)
            .ok_or(GraphOperationError::EdgeNotFound(edge_id))?;
        
        let graph = self.graph.lock().unwrap();
        let edge_data = graph.get_edge(*ctx_id)
            .ok_or(GraphOperationError::EdgeNotFound(edge_id))?;
        let edge = edge_data.0.clone();
        let source_ctx = edge_data.1;
        let target_ctx = edge_data.2;
        
        let source = self.reverse_node_map.get(&source_ctx)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        let target = self.reverse_node_map.get(&target_ctx)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        
        // Start with original metadata if available
        let mut metadata = self.edge_metadata.get(&edge_id)
            .cloned()
            .unwrap_or_else(|| edge.properties.clone());
        
        // Add/override concept-specific fields
        metadata.insert("strength".to_string(), serde_json::Value::from(edge.strength as f64));
        
        // Merge any additional properties from the edge
        for (key, value) in edge.properties {
            if !metadata.contains_key(&key) {
                metadata.insert(key, value);
            }
        }
        
        Ok((
            EdgeData {
                edge_type: edge.relationship_type.to_string(),
                metadata,
            },
            *source,
            *target,
        ))
    }
    
    fn list_nodes(&self) -> Vec<(NodeId, NodeData)> {
        self.node_id_map.iter()
            .filter_map(|(domain_id, ctx_id)| {
                self.graph.lock().unwrap().get_node(*ctx_id).cloned().map(|node| {
                    let position = Position3D {
                        x: node.position.coordinates.get(0).copied().unwrap_or(0.0) as f64,
                        y: node.position.coordinates.get(1).copied().unwrap_or(0.0) as f64,
                        z: node.position.coordinates.get(2).copied().unwrap_or(0.0) as f64,
                    };
                    
                    // Start with original metadata if available
                    let mut metadata = self.node_metadata.get(domain_id)
                        .cloned()
                        .unwrap_or_else(|| node.metadata.clone());
                    
                    // Add/override concept-specific fields
                    metadata.insert("label".to_string(), serde_json::Value::String(node.label.clone()));
                    metadata.insert("concept_id".to_string(), serde_json::Value::String(format!("{:?}", node.concept_id)));
                    
                    // Get original node type or default to "concept"
                    let node_type = self.node_types.get(domain_id)
                        .cloned()
                        .unwrap_or_else(|| "concept".to_string());
                    
                    (*domain_id, NodeData {
                        node_type,
                        position,
                        metadata,
                    })
                })
            })
            .collect()
    }
    
    fn list_edges(&self) -> Vec<(EdgeId, EdgeData, NodeId, NodeId)> {
        self.edge_id_map.iter()
            .filter_map(|(edge_id, ctx_id)| {
                let graph = self.graph.lock().unwrap();
                graph.get_edge(*ctx_id).and_then(|(edge, source_ctx, target_ctx)| {
                    let edge = edge.clone();
                    let source = self.reverse_node_map.get(&source_ctx)?;
                    let target = self.reverse_node_map.get(&target_ctx)?;
                    
                    // Start with original metadata if available
                    let mut metadata = self.edge_metadata.get(edge_id)
                        .cloned()
                        .unwrap_or_else(|| edge.properties.clone());
                    
                    // Add/override concept-specific fields
                    metadata.insert("strength".to_string(), serde_json::Value::from(edge.strength as f64));
                    
                    Some((
                        *edge_id,
                        EdgeData {
                            edge_type: edge.relationship_type.to_string(),
                            metadata,
                        },
                        *source,
                        *target,
                    ))
                })
            })
            .collect()
    }
    
    fn get_metadata(&self) -> GraphMetadata {
        GraphMetadata {
            name: format!("Concept Graph {}", self.graph_id),
            description: "Semantic graph with conceptual spaces".to_string(),
            properties: HashMap::new(),
        }
    }
    
    fn update_metadata(&mut self, _metadata: GraphMetadata) -> GraphResult<()> {
        // ConceptGraph doesn't support metadata updates
        Ok(())
    }
    
    fn find_nodes_by_type(&self, node_type: &str) -> Vec<NodeId> {
        if node_type == "concept" {
            self.node_id_map.keys().copied().collect()
        } else {
            // Check stored node types
            self.node_types.iter()
                .filter_map(|(node_id, stored_type)| {
                    if stored_type == node_type {
                        Some(*node_id)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
    
    fn find_edges_by_type(&self, edge_type: &str) -> Vec<EdgeId> {
        self.edge_id_map.iter()
            .filter_map(|(edge_id, ctx_id)| {
                let graph = self.graph.lock().unwrap();
                graph.get_edge(*ctx_id).and_then(|(edge, _, _)| {
                    if edge.relationship_type.to_string() == edge_type {
                        Some(*edge_id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
} 