//! Adapter for ContextGraph implementation

use crate::abstraction::{
    GraphImplementation, GraphMetadata, GraphOperationError, GraphResult,
    NodeData, EdgeData, Position3D,
};
use cim_contextgraph::ContextGraph;
use cim_domain::{NodeId, EdgeId, GraphId};
use std::collections::HashMap;

/// Adapter that wraps ContextGraph to implement GraphImplementation
#[derive(Clone)]
pub struct ContextGraphAdapter {
    graph: ContextGraph<serde_json::Value, serde_json::Value>,
    graph_id: GraphId,
    name: String,
    // Maps between domain IDs and ContextGraph IDs
    node_id_map: HashMap<NodeId, cim_contextgraph::NodeId>,
    edge_id_map: HashMap<EdgeId, cim_contextgraph::EdgeId>,
    // Reverse maps
    node_id_reverse: HashMap<cim_contextgraph::NodeId, NodeId>,
    edge_id_reverse: HashMap<cim_contextgraph::EdgeId, EdgeId>,
}

impl ContextGraphAdapter {
    /// Create a new adapter
    pub fn new(graph_id: GraphId, name: String) -> Self {
        let graph = ContextGraph::new(&name);
        
        Self {
            graph,
            graph_id,
            name,
            node_id_map: HashMap::new(),
            edge_id_map: HashMap::new(),
            node_id_reverse: HashMap::new(),
            edge_id_reverse: HashMap::new(),
        }
    }
}

impl GraphImplementation for ContextGraphAdapter {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
    
    fn add_node(&mut self, node_id: NodeId, data: NodeData) -> GraphResult<()> {
        // Convert NodeData to Value
        let value = serde_json::json!({
            "type": data.node_type,
            "position": {
                "x": data.position.x,
                "y": data.position.y,
                "z": data.position.z,
            },
            "metadata": data.metadata,
        });
        
        let ctx_id = self.graph.add_node(value);
        self.node_id_map.insert(node_id, ctx_id);
        self.node_id_reverse.insert(ctx_id, node_id);
        Ok(())
    }
    
    fn add_edge(&mut self, edge_id: EdgeId, source: NodeId, target: NodeId, data: EdgeData) -> GraphResult<()> {
        let from_ctx = self.node_id_map.get(&source)
            .ok_or(GraphOperationError::NodeNotFound(source))?;
        let to_ctx = self.node_id_map.get(&target)
            .ok_or(GraphOperationError::NodeNotFound(target))?;
        
        // Convert EdgeData to Value
        let value = serde_json::json!({
            "type": data.edge_type,
            "metadata": data.metadata,
        });
        
        match self.graph.add_edge(*from_ctx, *to_ctx, value) {
            Ok(ctx_edge_id) => {
                self.edge_id_map.insert(edge_id, ctx_edge_id);
                self.edge_id_reverse.insert(ctx_edge_id, edge_id);
                Ok(())
            }
            Err(e) => Err(GraphOperationError::EdgeCreationFailed(e.to_string())),
        }
    }
    
    fn get_node(&self, node_id: NodeId) -> GraphResult<NodeData> {
        let ctx_id = self.node_id_map.get(&node_id)
            .ok_or(GraphOperationError::NodeNotFound(node_id))?;
        
        let node = self.graph.get_node(*ctx_id)
            .ok_or(GraphOperationError::NodeNotFound(node_id))?;
        
        // Convert Value back to NodeData
        let node_type = node.value.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let position = if let Some(pos) = node.value.get("position") {
            Position3D {
                x: pos.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                y: pos.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                z: pos.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
            }
        } else {
            Position3D::default()
        };
        
        let metadata = node.value.get("metadata")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        
        Ok(NodeData {
            node_type,
            position,
            metadata,
        })
    }
    
    fn get_edge(&self, edge_id: EdgeId) -> GraphResult<(EdgeData, NodeId, NodeId)> {
        let ctx_id = self.edge_id_map.get(&edge_id)
            .ok_or(GraphOperationError::EdgeNotFound(edge_id))?;
        
        let edge = self.graph.get_edge(*ctx_id)
            .ok_or(GraphOperationError::EdgeNotFound(edge_id))?;
        
        // Convert Value back to EdgeData
        let edge_type = edge.value.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let metadata = edge.value.get("metadata")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        
        // Find source and target nodes
        let source = self.node_id_reverse.get(&edge.source)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        let target = self.node_id_reverse.get(&edge.target)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        
        Ok((
            EdgeData {
                edge_type,
                metadata,
            },
            *source,
            *target,
        ))
    }
    
    fn list_nodes(&self) -> Vec<(NodeId, NodeData)> {
        self.node_id_map.iter()
            .filter_map(|(domain_id, ctx_id)| {
                self.graph.get_node(*ctx_id).and_then(|node| {
                    // Convert Value to NodeData
                    let node_type = node.value.get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let position = if let Some(pos) = node.value.get("position") {
                        Position3D {
                            x: pos.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            y: pos.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            z: pos.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        }
                    } else {
                        Position3D::default()
                    };
                    
                    let metadata = node.value.get("metadata")
                        .and_then(|v| v.as_object())
                        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default();
                    
                    Some((*domain_id, NodeData {
                        node_type,
                        position,
                        metadata,
                    }))
                })
            })
            .collect()
    }
    
    fn list_edges(&self) -> Vec<(EdgeId, EdgeData, NodeId, NodeId)> {
        self.edge_id_map.iter()
            .filter_map(|(edge_id, ctx_id)| {
                self.graph.get_edge(*ctx_id).and_then(|edge| {
                    // Convert Value to EdgeData
                    let edge_type = edge.value.get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let metadata = edge.value.get("metadata")
                        .and_then(|v| v.as_object())
                        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default();
                    
                    // Find source and target
                    let source = self.node_id_reverse.get(&edge.source)?;
                    let target = self.node_id_reverse.get(&edge.target)?;
                    
                    Some((
                        *edge_id,
                        EdgeData {
                            edge_type,
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
            name: self.name.clone(),
            description: "ContextGraph adapter".to_string(),
            properties: HashMap::new(),
        }
    }
    
    fn update_metadata(&mut self, metadata: GraphMetadata) -> GraphResult<()> {
        self.name = metadata.name;
        // ContextGraph doesn't support full metadata updates
        Ok(())
    }
    
    fn find_nodes_by_type(&self, node_type: &str) -> Vec<NodeId> {
        self.node_id_map.iter()
            .filter_map(|(domain_id, ctx_id)| {
                self.graph.get_node(*ctx_id).and_then(|node| {
                    let nt = node.value.get("type")
                        .and_then(|v| v.as_str())?;
                    if nt == node_type {
                        Some(*domain_id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
    
    fn find_edges_by_type(&self, edge_type: &str) -> Vec<EdgeId> {
        self.edge_id_map.iter()
            .filter_map(|(edge_id, ctx_id)| {
                self.graph.get_edge(*ctx_id).and_then(|edge| {
                    let et = edge.value.get("type")
                        .and_then(|v| v.as_str())?;
                    if et == edge_type {
                        Some(*edge_id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
} 