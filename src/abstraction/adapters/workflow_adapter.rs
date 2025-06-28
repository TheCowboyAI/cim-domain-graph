//! Adapter for WorkflowGraph implementation

use crate::abstraction::{
    GraphImplementation, GraphMetadata, GraphOperationError, GraphResult,
    NodeData, EdgeData, Position3D,
};
use cim_domain::{NodeId, EdgeId, GraphId};
use cim_workflow_graph::WorkflowGraph;
use cim_domain_workflow::value_objects::{StepId, StepType};
use std::collections::HashMap;

/// Adapter that wraps WorkflowGraph to implement GraphImplementation
#[derive(Clone)]
pub struct WorkflowGraphAdapter {
    graph: WorkflowGraph,
    graph_id: GraphId,
    // Map domain NodeId to WorkflowGraph's StepId
    node_to_step: HashMap<NodeId, StepId>,
    step_to_node: HashMap<StepId, NodeId>,
    // Map EdgeId to dependency relationships
    edge_map: HashMap<EdgeId, (StepId, StepId)>,
    // Store original metadata to preserve it
    node_metadata: HashMap<NodeId, HashMap<String, serde_json::Value>>,
    edge_metadata: HashMap<EdgeId, HashMap<String, serde_json::Value>>,
    // Store original positions
    node_positions: HashMap<NodeId, Position3D>,
    // Store original edge types
    edge_types: HashMap<EdgeId, String>,
}

impl WorkflowGraphAdapter {
    /// Create a new adapter
    pub fn new(graph_id: GraphId, name: &str) -> Self {
        let graph = WorkflowGraph::new(name.to_string(), "Graph adapter".to_string())
            .expect("Failed to create WorkflowGraph");
        
        Self {
            graph,
            graph_id,
            node_to_step: HashMap::new(),
            step_to_node: HashMap::new(),
            edge_map: HashMap::new(),
            node_metadata: HashMap::new(),
            edge_metadata: HashMap::new(),
            node_positions: HashMap::new(),
            edge_types: HashMap::new(),
        }
    }
}

impl GraphImplementation for WorkflowGraphAdapter {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
    
    fn add_node(&mut self, node_id: NodeId, data: NodeData) -> GraphResult<()> {
        // Store original metadata and position
        self.node_metadata.insert(node_id, data.metadata.clone());
        self.node_positions.insert(node_id, data.position.clone());
        
        // Convert node type to StepType
        let step_type = match data.node_type.as_str() {
            "manual" => StepType::Manual,
            "automated" => StepType::Automated,
            "decision" => StepType::Decision,
            "approval" => StepType::Approval,
            "integration" => StepType::Integration,
            "parallel" => StepType::Parallel,
            _ => StepType::Custom(data.node_type.clone()),
        };
        
        // Extract dependencies if any were pre-specified
        let dependencies = data.metadata.get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| {
                        // Try to find the StepId for this dependency
                        self.node_to_step.iter()
                            .find(|(_, step_id)| step_id.as_uuid().to_string() == s)
                            .map(|(_, step_id)| *step_id)
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        // Add step to workflow
        let step_id = self.graph.add_step(
            data.metadata.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Step")
                .to_string(),
            data.metadata.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            step_type,
            HashMap::new(), // config
            dependencies,
            data.metadata.get("estimated_duration_minutes")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            data.metadata.get("assigned_to")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        ).map_err(|e| GraphOperationError::NodeCreationFailed(e.to_string()))?;
        
        // Store mapping
        self.node_to_step.insert(node_id, step_id);
        self.step_to_node.insert(step_id, node_id);
        
        Ok(())
    }
    
    fn add_edge(&mut self, edge_id: EdgeId, source: NodeId, target: NodeId, data: EdgeData) -> GraphResult<()> {
        // Store original metadata and edge type
        self.edge_metadata.insert(edge_id, data.metadata.clone());
        self.edge_types.insert(edge_id, data.edge_type.clone());
        
        let source_step = self.node_to_step.get(&source)
            .ok_or_else(|| GraphOperationError::NodeNotFound(source))?;
        let target_step = self.node_to_step.get(&target)
            .ok_or_else(|| GraphOperationError::NodeNotFound(target))?;
        
        // WorkflowGraph doesn't support adding dependencies after step creation
        // We'll just store the edge mapping for retrieval
        self.edge_map.insert(edge_id, (*source_step, *target_step));
        
        Ok(())
    }
    
    fn get_node(&self, node_id: NodeId) -> GraphResult<NodeData> {
        let step_id = self.node_to_step.get(&node_id)
            .ok_or_else(|| GraphOperationError::NodeNotFound(node_id))?;
        
        let step = self.graph.workflow.steps.get(step_id)
            .ok_or_else(|| GraphOperationError::NodeNotFound(node_id))?;
        
        // Start with original metadata if available
        let mut metadata = self.node_metadata.get(&node_id)
            .cloned()
            .unwrap_or_default();
        
        // Override/add workflow-specific fields
        metadata.insert("name".to_string(), serde_json::Value::String(step.name.clone()));
        metadata.insert("description".to_string(), serde_json::Value::String(step.description.clone()));
        metadata.insert("status".to_string(), serde_json::Value::String(format!("{:?}", step.status)));
        metadata.insert("step_id".to_string(), serde_json::Value::String(step_id.as_uuid().to_string()));
        
        if let Some(duration) = step.estimated_duration_minutes {
            metadata.insert("estimated_duration_minutes".to_string(), serde_json::Value::from(duration));
        }
        
        if let Some(ref assigned) = step.assigned_to {
            metadata.insert("assigned_to".to_string(), serde_json::Value::String(assigned.clone()));
        }
        
        // Get original position or default
        let position = self.node_positions.get(&node_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(NodeData {
            node_type: match &step.step_type {
                StepType::Manual => "manual".to_string(),
                StepType::Automated => "automated".to_string(),
                StepType::Decision => "decision".to_string(),
                StepType::Approval => "approval".to_string(),
                StepType::Integration => "integration".to_string(),
                StepType::Parallel => "parallel".to_string(),
                StepType::Custom(name) => name.clone(),
            },
            position,
            metadata,
        })
    }
    
    fn get_edge(&self, edge_id: EdgeId) -> GraphResult<(EdgeData, NodeId, NodeId)> {
        let (source_step, target_step) = self.edge_map.get(&edge_id)
            .ok_or_else(|| GraphOperationError::EdgeNotFound(edge_id))?;
        
        let source_node = self.step_to_node.get(source_step)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        let target_node = self.step_to_node.get(target_step)
            .ok_or_else(|| GraphOperationError::NodeNotFound(NodeId::new()))?;
        
        // Get original metadata or empty
        let metadata = self.edge_metadata.get(&edge_id)
            .cloned()
            .unwrap_or_default();
        
        // Get original edge type or default to "dependency"
        let edge_type = self.edge_types.get(&edge_id)
            .cloned()
            .unwrap_or_else(|| "dependency".to_string());
        
        Ok((
            EdgeData {
                edge_type,
                metadata,
            },
            *source_node,
            *target_node,
        ))
    }
    
    fn list_nodes(&self) -> Vec<(NodeId, NodeData)> {
        self.graph.workflow.steps.iter()
            .filter_map(|(step_id, step)| {
                self.step_to_node.get(step_id).map(|node_id| {
                    // Start with original metadata if available
                    let mut metadata = self.node_metadata.get(node_id)
                        .cloned()
                        .unwrap_or_default();
                    
                    // Override/add workflow-specific fields
                    metadata.insert("name".to_string(), serde_json::Value::String(step.name.clone()));
                    metadata.insert("description".to_string(), serde_json::Value::String(step.description.clone()));
                    metadata.insert("status".to_string(), serde_json::Value::String(format!("{:?}", step.status)));
                    metadata.insert("step_id".to_string(), serde_json::Value::String(step_id.as_uuid().to_string()));
                    
                    if let Some(duration) = step.estimated_duration_minutes {
                        metadata.insert("estimated_duration_minutes".to_string(), serde_json::Value::from(duration));
                    }
                    
                    if let Some(ref assigned) = step.assigned_to {
                        metadata.insert("assigned_to".to_string(), serde_json::Value::String(assigned.clone()));
                    }
                    
                    // Get original position or default
                    let position = self.node_positions.get(node_id)
                        .cloned()
                        .unwrap_or_default();
                    
                    (*node_id, NodeData {
                        node_type: match &step.step_type {
                            StepType::Manual => "manual".to_string(),
                            StepType::Automated => "automated".to_string(),
                            StepType::Decision => "decision".to_string(),
                            StepType::Approval => "approval".to_string(),
                            StepType::Integration => "integration".to_string(),
                            StepType::Parallel => "parallel".to_string(),
                            StepType::Custom(name) => name.clone(),
                        },
                        position,
                        metadata,
                    })
                })
            })
            .collect()
    }
    
    fn list_edges(&self) -> Vec<(EdgeId, EdgeData, NodeId, NodeId)> {
        self.edge_map.iter()
            .filter_map(|(edge_id, (source_step, target_step))| {
                let source_node = self.step_to_node.get(source_step)?;
                let target_node = self.step_to_node.get(target_step)?;
                
                // Get original metadata or empty
                let metadata = self.edge_metadata.get(edge_id)
                    .cloned()
                    .unwrap_or_default();
                
                // Get original edge type or default to "dependency"
                let edge_type = self.edge_types.get(edge_id)
                    .cloned()
                    .unwrap_or_else(|| "dependency".to_string());
                
                Some((
                    *edge_id,
                    EdgeData {
                        edge_type,
                        metadata,
                    },
                    *source_node,
                    *target_node,
                ))
            })
            .collect()
    }
    
    fn get_metadata(&self) -> GraphMetadata {
        GraphMetadata {
            name: self.graph.metadata.name.clone(),
            description: self.graph.metadata.description.clone(),
            properties: self.graph.metadata.properties.clone(),
        }
    }
    
    fn update_metadata(&mut self, metadata: GraphMetadata) -> GraphResult<()> {
        self.graph.metadata.name = metadata.name;
        self.graph.metadata.description = metadata.description;
        self.graph.metadata.properties = metadata.properties;
        Ok(())
    }
    
    fn find_nodes_by_type(&self, node_type: &str) -> Vec<NodeId> {
        let target_type = match node_type {
            "manual" => StepType::Manual,
            "automated" => StepType::Automated,
            "decision" => StepType::Decision,
            "approval" => StepType::Approval,
            "integration" => StepType::Integration,
            "parallel" => StepType::Parallel,
            other => StepType::Custom(other.to_string()),
        };
        
        self.graph.find_steps_by_type(target_type).into_iter()
            .filter_map(|step_id| self.step_to_node.get(&step_id).copied())
            .collect()
    }
    
    fn find_edges_by_type(&self, edge_type: &str) -> Vec<EdgeId> {
        if edge_type == "dependency" {
            self.edge_map.keys().copied().collect()
        } else {
            Vec::new()
        }
    }
} 