//! Validation rules for deployment graphs

use crate::aggregate::business_graph::Graph;
use super::{DeploymentNodeType, DeploymentEdgeType, graph_adapter::DeploymentGraphExt};
use thiserror::Error;
use std::collections::{HashMap, HashSet, VecDeque};

/// Errors that can occur during deployment graph validation
#[derive(Debug, Error)]
pub enum DeploymentError {
    #[error("Cyclic dependency detected: {0}")]
    CyclicDependency(String),
    
    #[error("Missing required dependency: {0} requires {1}")]
    MissingDependency(String, String),
    
    #[error("Invalid node configuration: {0}")]
    InvalidNodeConfig(String),
    
    #[error("Port conflict: {port} is used by multiple services: {services:?}")]
    PortConflict { port: u16, services: Vec<String> },
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Invalid edge: {0}")]
    InvalidEdge(String),
    
    #[error("Orphaned node: {0} has no connections")]
    OrphanedNode(String),
    
    #[error("Storage conflict: {path} is mounted by multiple services")]
    StorageConflict { path: String },
}

/// Validate a deployment graph for correctness
pub fn validate_deployment_graph(graph: &Graph) -> Result<(), DeploymentError> {
    // Run all validation checks
    check_for_cycles(graph)?;
    check_dependencies(graph)?;
    check_port_conflicts(graph)?;
    check_resource_limits(graph)?;
    check_storage_conflicts(graph)?;
    check_node_configurations(graph)?;
    
    Ok(())
}

/// Check for cyclic dependencies in the graph
fn check_for_cycles(graph: &Graph) -> Result<(), DeploymentError> {
    let nodes = graph.get_all_nodes();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    
    for node in &nodes {
        if !visited.contains(&node.id) {
            if has_cycle_dfs(graph, &node.id, &mut visited, &mut rec_stack)? {
                return Err(DeploymentError::CyclicDependency(
                    "Deployment graph contains circular dependencies".to_string()
                ));
            }
        }
    }
    
    Ok(())
}

/// DFS helper for cycle detection
fn has_cycle_dfs(
    graph: &Graph,
    node_id: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> Result<bool, DeploymentError> {
    visited.insert(node_id.to_string());
    rec_stack.insert(node_id.to_string());
    
    // Get all outgoing edges that represent dependencies
    let edges = graph.get_edges_from(node_id);
    for edge in edges {
        if let Ok(edge_data) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
            if edge_data.is_startup_dependency() {
                if !visited.contains(&edge.to) {
                    if has_cycle_dfs(graph, &edge.to, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&edge.to) {
                    return Ok(true);
                }
            }
        }
    }
    
    rec_stack.remove(node_id);
    Ok(false)
}

/// Check that all required dependencies exist
fn check_dependencies(graph: &Graph) -> Result<(), DeploymentError> {
    let node_ids: HashSet<String> = graph.get_all_nodes().iter().map(|n| n.id.clone()).collect();
    
    for edge in graph.get_all_edges() {
        if let Ok(edge_data) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
            match edge_data {
                DeploymentEdgeType::DependsOn { required: true, .. } => {
                    if !node_ids.contains(&edge.to) {
                        return Err(DeploymentError::MissingDependency(
                            edge.from.clone(),
                            edge.to.clone(),
                        ));
                    }
                }
                DeploymentEdgeType::LoadBalances { .. } => {
                    if !node_ids.contains(&edge.to) {
                        return Err(DeploymentError::MissingDependency(
                            edge.from.clone(),
                            edge.to.clone(),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}

/// Check for port conflicts between services
fn check_port_conflicts(graph: &Graph) -> Result<(), DeploymentError> {
    let mut port_usage: HashMap<u16, Vec<String>> = HashMap::new();
    
    for node in graph.get_all_nodes() {
        if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
            for port in node_type.exposed_ports() {
                port_usage.entry(port).or_default().push(node_type.name().to_string());
            }
        }
    }
    
    // Check for conflicts
    for (port, services) in port_usage {
        if services.len() > 1 {
            return Err(DeploymentError::PortConflict { port, services });
        }
    }
    
    Ok(())
}

/// Check resource limits
fn check_resource_limits(graph: &Graph) -> Result<(), DeploymentError> {
    let mut total_cpu = 0.0;
    let mut total_memory_mb = 0;
    let mut total_disk_gb = 0;
    
    for node in graph.get_all_nodes() {
        if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
            if let Some(resources) = node_type.resources() {
                total_cpu += resources.cpu_cores.unwrap_or(0.0);
                total_memory_mb += resources.memory_mb.unwrap_or(0);
                total_disk_gb += resources.disk_gb.unwrap_or(0);
            }
        }
    }
    
    // These limits would typically come from configuration
    const MAX_CPU_CORES: f32 = 64.0;
    const MAX_MEMORY_MB: u32 = 128_000; // 128GB
    const MAX_DISK_GB: u32 = 10_000; // 10TB
    
    if total_cpu > MAX_CPU_CORES {
        return Err(DeploymentError::ResourceLimitExceeded(
            format!("Total CPU cores ({}) exceeds limit ({})", total_cpu, MAX_CPU_CORES)
        ));
    }
    
    if total_memory_mb > MAX_MEMORY_MB {
        return Err(DeploymentError::ResourceLimitExceeded(
            format!("Total memory ({}MB) exceeds limit ({}MB)", total_memory_mb, MAX_MEMORY_MB)
        ));
    }
    
    if total_disk_gb > MAX_DISK_GB {
        return Err(DeploymentError::ResourceLimitExceeded(
            format!("Total disk ({}GB) exceeds limit ({}GB)", total_disk_gb, MAX_DISK_GB)
        ));
    }
    
    Ok(())
}

/// Check for storage mount conflicts
fn check_storage_conflicts(graph: &Graph) -> Result<(), DeploymentError> {
    let mut mount_paths: HashMap<String, Vec<String>> = HashMap::new();
    
    for edge in graph.get_all_edges() {
        if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
            if let DeploymentEdgeType::MountsVolume { mount_path, read_only: false } = edge_type {
                mount_paths.entry(mount_path.clone()).or_default().push(edge.from.clone());
            }
        }
    }
    
    // Check for write conflicts
    for (path, services) in mount_paths {
        if services.len() > 1 {
            return Err(DeploymentError::StorageConflict { path });
        }
    }
    
    Ok(())
}

/// Check individual node configurations
fn check_node_configurations(graph: &Graph) -> Result<(), DeploymentError> {
    for node in graph.get_all_nodes() {
        if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
            match node_type {
                DeploymentNodeType::Service { name, command, .. } => {
                    if name.is_empty() {
                        return Err(DeploymentError::InvalidNodeConfig("Service name cannot be empty".to_string()));
                    }
                    if command.is_empty() {
                        return Err(DeploymentError::InvalidNodeConfig(
                            format!("Service '{}' has empty command", name)
                        ));
                    }
                }
                DeploymentNodeType::Database { name, version, .. } => {
                    if version.is_empty() {
                        return Err(DeploymentError::InvalidNodeConfig(
                            format!("Database '{}' has no version specified", name)
                        ));
                    }
                }
                DeploymentNodeType::MessageBus { cluster_size, .. } => {
                    if cluster_size == 0 {
                        return Err(DeploymentError::InvalidNodeConfig(
                            "Message bus cluster size must be at least 1".to_string()
                        ));
                    }
                }
                DeploymentNodeType::LoadBalancer { backends, .. } => {
                    if backends.is_empty() {
                        return Err(DeploymentError::InvalidNodeConfig(
                            "Load balancer must have at least one backend".to_string()
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}

/// Get topological order of nodes for deployment
pub fn get_deployment_order(graph: &Graph) -> Result<Vec<String>, DeploymentError> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let nodes = graph.get_all_nodes();
    
    // Initialize in-degree for all nodes
    for node in &nodes {
        in_degree.insert(node.id.clone(), 0);
    }
    
    // Calculate in-degrees based on dependency edges
    for edge in graph.get_all_edges() {
        if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
            if edge_type.is_startup_dependency() {
                *in_degree.get_mut(&edge.to).unwrap() += 1;
            }
        }
    }
    
    // Topological sort using Kahn's algorithm
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    
    // Find all nodes with no incoming edges
    for (node_id, degree) in &in_degree {
        if *degree == 0 {
            queue.push_back(node_id.clone());
        }
    }
    
    while let Some(node_id) = queue.pop_front() {
        result.push(node_id.clone());
        
        // Reduce in-degree of neighbors
        for edge in graph.get_edges_from(&node_id) {
            if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
                if edge_type.is_startup_dependency() {
                    let neighbor_degree = in_degree.get_mut(&edge.to).unwrap();
                    *neighbor_degree -= 1;
                    if *neighbor_degree == 0 {
                        queue.push_back(edge.to.clone());
                    }
                }
            }
        }
    }
    
    if result.len() != nodes.len() {
        return Err(DeploymentError::CyclicDependency(
            "Cannot determine deployment order due to circular dependencies".to_string()
        ));
    }
    
    Ok(result)
}