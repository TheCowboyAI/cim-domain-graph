//! Adapter for IpldGraph implementation

use crate::abstraction::{
    GraphImplementation, GraphMetadata, GraphOperationError, GraphResult,
    NodeData, EdgeData, Position3D,
};
use cim_domain::{NodeId, EdgeId, GraphId};
use cim_ipld_graph::{CidDag, EventNode, ObjectNode};
use cid::Cid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Type of IPLD node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldNodeType {
    Event(EventNode),
    Object(ObjectNode),
}

/// Adapter that wraps IpldGraph (CidDag) to implement GraphImplementation
#[derive(Clone)]
pub struct IpldGraphAdapter {
    dag: CidDag<IpldNodeType>,
    graph_id: GraphId,
    // Map domain NodeId to CID
    node_to_cid: HashMap<NodeId, Cid>,
    cid_to_node: HashMap<Cid, NodeId>,
    // Map EdgeId to edge info
    edge_map: HashMap<EdgeId, (Cid, Cid, String)>,
}

impl IpldGraphAdapter {
    /// Create a new adapter
    pub fn new(graph_id: GraphId) -> Self {
        Self {
            dag: CidDag::new(),
            graph_id,
            node_to_cid: HashMap::new(),
            cid_to_node: HashMap::new(),
            edge_map: HashMap::new(),
        }
    }
    
    /// Generate a CID from content
    fn generate_cid(content: &[u8]) -> Cid {
        let hash = blake3::hash(content);
        let hash_bytes = hash.as_bytes();
        
        // Create multihash manually with BLAKE3 code (0x1e)
        let code = 0x1e; // BLAKE3-256
        let size = hash_bytes.len() as u8;
        
        let mut multihash_bytes = Vec::new();
        multihash_bytes.push(code);
        multihash_bytes.push(size);
        multihash_bytes.extend_from_slice(hash_bytes);
        
        let mh = multihash::Multihash::from_bytes(&multihash_bytes).unwrap();
        Cid::new_v1(0x55, mh) // 0x55 is raw codec
    }
}

impl GraphImplementation for IpldGraphAdapter {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
    
    fn add_node(&mut self, node_id: NodeId, data: NodeData) -> GraphResult<()> {
        // Create an IPLD node based on the node type
        let ipld_node = match data.node_type.as_str() {
            "event" => IpldNodeType::Event(EventNode {
                event_id: data.metadata.get("event_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&node_id.to_string())
                    .to_string(),
                aggregate_id: data.metadata.get("aggregate_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                event_type: data.metadata.get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                sequence: data.metadata.get("sequence")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                payload_cid: data.metadata.get("payload_cid")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<Cid>().ok()),
            }),
            _ => IpldNodeType::Object(ObjectNode {
                object_type: data.node_type.clone(),
                size: data.metadata.get("size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                mime_type: data.metadata.get("mime_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                chunks: data.metadata.get("chunks")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .filter_map(|s| s.parse::<Cid>().ok())
                            .collect()
                    })
                    .unwrap_or_default(),
            }),
        };
        
        // Generate CID for this node
        let cid = Self::generate_cid(node_id.to_string().as_bytes());
        
        // Get timestamp
        let timestamp = data.metadata.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });
        
        // Add to DAG
        self.dag.add_node(cid, ipld_node, timestamp)
            .map_err(|e| GraphOperationError::NodeCreationFailed(e.to_string()))?;
        
        // Store mapping
        self.node_to_cid.insert(node_id, cid);
        self.cid_to_node.insert(cid, node_id);
        
        Ok(())
    }
    
    fn add_edge(&mut self, edge_id: EdgeId, source: NodeId, target: NodeId, data: EdgeData) -> GraphResult<()> {
        let source_cid = self.node_to_cid.get(&source)
            .ok_or_else(|| GraphOperationError::NodeNotFound(source))?;
        let target_cid = self.node_to_cid.get(&target)
            .ok_or_else(|| GraphOperationError::NodeNotFound(target))?;
        
        // Add edge based on type
        match data.edge_type.as_str() {
            "causal" => {
                self.dag.add_causal_edge(*source_cid, *target_cid)
                    .map_err(|e| GraphOperationError::EdgeCreationFailed(e.to_string()))?;
            }
            "reference" => {
                self.dag.add_reference(*source_cid, *target_cid)
                    .map_err(|e| GraphOperationError::EdgeCreationFailed(e.to_string()))?;
            }
            _ => {
                // Default to causal
                self.dag.add_causal_edge(*source_cid, *target_cid)
                    .map_err(|e| GraphOperationError::EdgeCreationFailed(e.to_string()))?;
            }
        }
        
        // Store edge mapping
        self.edge_map.insert(edge_id, (*source_cid, *target_cid, data.edge_type));
        
        Ok(())
    }
    
    fn get_node(&self, node_id: NodeId) -> GraphResult<NodeData> {
        let cid = self.node_to_cid.get(&node_id)
            .ok_or_else(|| GraphOperationError::NodeNotFound(node_id))?;
        
        let cid_node = self.dag.get_node(cid)
            .ok_or_else(|| GraphOperationError::NodeNotFound(node_id))?;
        
        // Convert IpldNodeType to NodeData
        let (node_type, mut metadata) = match &cid_node.content {
            IpldNodeType::Event(event_node) => {
                let mut metadata = HashMap::new();
                metadata.insert("event_id".to_string(), serde_json::Value::String(event_node.event_id.clone()));
                metadata.insert("aggregate_id".to_string(), serde_json::Value::String(event_node.aggregate_id.clone()));
                metadata.insert("event_type".to_string(), serde_json::Value::String(event_node.event_type.clone()));
                metadata.insert("sequence".to_string(), serde_json::Value::from(event_node.sequence));
                if let Some(ref payload_cid) = event_node.payload_cid {
                    metadata.insert("payload_cid".to_string(), serde_json::Value::String(payload_cid.to_string()));
                }
                ("event".to_string(), metadata)
            }
            IpldNodeType::Object(object_node) => {
                let mut metadata = HashMap::new();
                metadata.insert("size".to_string(), serde_json::Value::from(object_node.size));
                if let Some(ref mime_type) = object_node.mime_type {
                    metadata.insert("mime_type".to_string(), serde_json::Value::String(mime_type.clone()));
                }
                if !object_node.chunks.is_empty() {
                    let chunks: Vec<serde_json::Value> = object_node.chunks.iter()
                        .map(|cid| serde_json::Value::String(cid.to_string()))
                        .collect();
                    metadata.insert("chunks".to_string(), serde_json::Value::Array(chunks));
                }
                (object_node.object_type.clone(), metadata)
            }
        };
        
        // Add CID and timestamp to metadata
        metadata.insert("cid".to_string(), serde_json::Value::String(cid.to_string()));
        metadata.insert("timestamp".to_string(), serde_json::Value::from(cid_node.timestamp));
        
        // Merge any additional metadata from the node
        for (k, v) in &cid_node.metadata {
            metadata.insert(k.clone(), v.clone());
        }
        
        Ok(NodeData {
            node_type,
            position: Position3D::default(), // IPLD graphs don't have positions
            metadata,
        })
    }
    
    fn get_edge(&self, edge_id: EdgeId) -> GraphResult<(EdgeData, NodeId, NodeId)> {
        let (source_cid, target_cid, edge_type) = self.edge_map.get(&edge_id)
            .ok_or_else(|| GraphOperationError::EdgeNotFound(edge_id))?;
        
        let source_node = self.cid_to_node.get(source_cid)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        let target_node = self.cid_to_node.get(target_cid)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        
        Ok((
            EdgeData {
                edge_type: edge_type.clone(),
                metadata: HashMap::new(),
            },
            *source_node,
            *target_node,
        ))
    }
    
    fn list_nodes(&self) -> Vec<(NodeId, NodeData)> {
        self.node_to_cid.iter()
            .filter_map(|(node_id, _cid)| {
                self.get_node(*node_id).ok().map(|data| (*node_id, data))
            })
            .collect()
    }
    
    fn list_edges(&self) -> Vec<(EdgeId, EdgeData, NodeId, NodeId)> {
        self.edge_map.iter()
            .filter_map(|(edge_id, (source_cid, target_cid, edge_type))| {
                let source_node = self.cid_to_node.get(source_cid)?;
                let target_node = self.cid_to_node.get(target_cid)?;
                
                Some((
                    *edge_id,
                    EdgeData {
                        edge_type: edge_type.clone(),
                        metadata: HashMap::new(),
                    },
                    *source_node,
                    *target_node,
                ))
            })
            .collect()
    }
    
    fn get_metadata(&self) -> GraphMetadata {
        let mut properties = HashMap::new();
        properties.insert("graph_type".to_string(), serde_json::Value::String("ipld_dag".to_string()));
        properties.insert("root_count".to_string(), serde_json::Value::from(self.dag.root_cids().len()));
        properties.insert("leaf_count".to_string(), serde_json::Value::from(self.dag.latest_cids().len()));
        
        GraphMetadata {
            name: format!("IPLD Graph {}", self.graph_id),
            description: "Content-addressed DAG for Event Store and Object Store".to_string(),
            properties,
        }
    }
    
    fn update_metadata(&mut self, _metadata: GraphMetadata) -> GraphResult<()> {
        // IPLD graphs don't support metadata updates
        Err(GraphOperationError::NotSupported("IPLD graphs don't support metadata updates".to_string()))
    }
    
    fn find_nodes_by_type(&self, node_type: &str) -> Vec<NodeId> {
        self.node_to_cid.iter()
            .filter_map(|(node_id, cid)| {
                self.dag.get_node(cid).and_then(|cid_node| {
                    let nt = match &cid_node.content {
                        IpldNodeType::Event(_) => "event",
                        IpldNodeType::Object(obj) => &obj.object_type,
                    };
                    if nt == node_type {
                        Some(*node_id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
    
    fn find_edges_by_type(&self, edge_type: &str) -> Vec<EdgeId> {
        self.edge_map.iter()
            .filter_map(|(edge_id, (_, _, et))| {
                if et == edge_type {
                    Some(*edge_id)
                } else {
                    None
                }
            })
            .collect()
    }
} 