//! Query systems for graph operations

use bevy_ecs::prelude::*;
use crate::components::{NodeEntity, EdgeEntity, GraphEntity, NodeType};
use crate::{NodeId, EdgeId, GraphId};
use std::collections::{HashMap, HashSet, VecDeque};
use petgraph::graph::{UnGraph, NodeIndex};
use petgraph::algo::dijkstra;

/// Event for node type query requests
#[derive(Event, Debug, Clone)]
pub struct FindNodesByTypeRequest {
    pub graph_id: GraphId,
    pub node_type: NodeType,
}

/// Event for node type query responses
#[derive(Event, Debug, Clone)]
pub struct FindNodesByTypeResponse {
    pub graph_id: GraphId,
    pub node_type: NodeType,
    pub nodes: Vec<NodeId>,
}

/// Find nodes by type
pub fn find_nodes_by_type_system(
    mut requests: EventReader<FindNodesByTypeRequest>,
    mut responses: EventWriter<FindNodesByTypeResponse>,
    node_query: Query<(&NodeEntity, &Parent)>,
    graph_query: Query<&GraphEntity>,
) {
    for request in requests.read() {
        let mut matching_nodes = Vec::new();
        
        // Find all nodes in the specified graph with the requested type
        for (node, parent) in &node_query {
            // Check if parent is the requested graph
            if let Ok(graph) = graph_query.get(parent.get()) {
                if graph.graph_id == request.graph_id && node.node_type == request.node_type {
                    matching_nodes.push(node.node_id);
                }
            }
        }
        
        // Send response
        responses.send(FindNodesByTypeResponse {
            graph_id: request.graph_id,
            node_type: request.node_type.clone(),
            nodes: matching_nodes,
        });
    }
}

/// Event for connected nodes query requests
#[derive(Event, Debug, Clone)]
pub struct FindConnectedNodesRequest {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub max_depth: Option<usize>,
}

/// Event for connected nodes query responses
#[derive(Event, Debug, Clone)]
pub struct FindConnectedNodesResponse {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub connected_nodes: HashMap<NodeId, usize>, // NodeId -> depth
}

/// Find connected nodes using BFS
pub fn find_connected_nodes_system(
    mut requests: EventReader<FindConnectedNodesRequest>,
    mut responses: EventWriter<FindConnectedNodesResponse>,
    node_query: Query<(&NodeEntity, &Parent)>,
    edge_query: Query<(&EdgeEntity, &Parent)>,
    graph_query: Query<&GraphEntity>,
) {
    for request in requests.read() {
        let mut connected_nodes = HashMap::new();
        let max_depth = request.max_depth.unwrap_or(usize::MAX);
        
        // Build adjacency list for the graph
        let mut adjacency: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        
        // First, collect all nodes in the graph
        for (node, parent) in &node_query {
            if let Ok(graph) = graph_query.get(parent.get()) {
                if graph.graph_id == request.graph_id {
                    adjacency.insert(node.node_id, HashSet::new());
                }
            }
        }
        
        // Then, build edges
        for (edge, parent) in &edge_query {
            if let Ok(graph) = graph_query.get(parent.get()) {
                if graph.graph_id == request.graph_id {
                    // Add bidirectional connections
                    if let Some(neighbors) = adjacency.get_mut(&edge.source) {
                        neighbors.insert(edge.target);
                    }
                    if let Some(neighbors) = adjacency.get_mut(&edge.target) {
                        neighbors.insert(edge.source);
                    }
                }
            }
        }
        
        // BFS to find connected nodes
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        if adjacency.contains_key(&request.node_id) {
            queue.push_back((request.node_id, 0));
            visited.insert(request.node_id);
            
            while let Some((current_node, depth)) = queue.pop_front() {
                if depth <= max_depth {
                    connected_nodes.insert(current_node, depth);
                    
                    if let Some(neighbors) = adjacency.get(&current_node) {
                        for &neighbor in neighbors {
                            if !visited.contains(&neighbor) && depth < max_depth {
                                visited.insert(neighbor);
                                queue.push_back((neighbor, depth + 1));
                            }
                        }
                    }
                }
            }
        }
        
        // Send response
        responses.send(FindConnectedNodesResponse {
            graph_id: request.graph_id,
            node_id: request.node_id,
            connected_nodes,
        });
    }
}

/// Event for shortest path query requests
#[derive(Event, Debug, Clone)]
pub struct ShortestPathRequest {
    pub graph_id: GraphId,
    pub source: NodeId,
    pub target: NodeId,
}

/// Event for shortest path query responses
#[derive(Event, Debug, Clone)]
pub struct ShortestPathResponse {
    pub graph_id: GraphId,
    pub source: NodeId,
    pub target: NodeId,
    pub path: Option<Vec<NodeId>>,
    pub distance: Option<usize>,
}

/// Calculate shortest path between nodes using Dijkstra's algorithm
pub fn calculate_shortest_path_system(
    mut requests: EventReader<ShortestPathRequest>,
    mut responses: EventWriter<ShortestPathResponse>,
    node_query: Query<(&NodeEntity, &Parent)>,
    edge_query: Query<(&EdgeEntity, &Parent)>,
    graph_query: Query<&GraphEntity>,
) {
    for request in requests.read() {
        // Build petgraph for pathfinding
        let mut graph = UnGraph::<NodeId, ()>::new_undirected();
        let mut node_indices: HashMap<NodeId, NodeIndex> = HashMap::new();
        let mut index_to_node: HashMap<NodeIndex, NodeId> = HashMap::new();
        
        // Add nodes
        for (node, parent) in &node_query {
            if let Ok(graph_entity) = graph_query.get(parent.get()) {
                if graph_entity.graph_id == request.graph_id {
                    let idx = graph.add_node(node.node_id);
                    node_indices.insert(node.node_id, idx);
                    index_to_node.insert(idx, node.node_id);
                }
            }
        }
        
        // Add edges
        for (edge, parent) in &edge_query {
            if let Ok(graph_entity) = graph_query.get(parent.get()) {
                if graph_entity.graph_id == request.graph_id {
                    if let (Some(&source_idx), Some(&target_idx)) = 
                        (node_indices.get(&edge.source), node_indices.get(&edge.target)) {
                        graph.add_edge(source_idx, target_idx, ());
                    }
                }
            }
        }
        
        // Calculate shortest path
        let mut path = None;
        let mut distance = None;
        
        if let (Some(&source_idx), Some(&target_idx)) = 
            (node_indices.get(&request.source), node_indices.get(&request.target)) {
            
            // Run Dijkstra's algorithm
            let node_map = dijkstra(&graph, source_idx, Some(target_idx), |_| 1);
            
            if let Some(&dist) = node_map.get(&target_idx) {
                distance = Some(dist as usize);
                
                // Reconstruct path
                let mut current = target_idx;
                let mut path_indices = vec![current];
                
                while current != source_idx {
                    // Find predecessor
                    let mut found = false;
                    for neighbor in graph.neighbors(current) {
                        if let Some(&neighbor_dist) = node_map.get(&neighbor) {
                            if neighbor_dist == (node_map[&current] - 1) {
                                path_indices.push(neighbor);
                                current = neighbor;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        break;
                    }
                }
                
                path_indices.reverse();
                path = Some(path_indices.into_iter()
                    .filter_map(|idx| index_to_node.get(&idx).cloned())
                    .collect());
            }
        }
        
        // Send response
        responses.send(ShortestPathResponse {
            graph_id: request.graph_id,
            source: request.source,
            target: request.target,
            path,
            distance,
        });
    }
}

/// Plugin to register query systems
pub struct GraphQueryPlugin;

impl bevy_app::Plugin for GraphQueryPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            .add_event::<FindNodesByTypeRequest>()
            .add_event::<FindNodesByTypeResponse>()
            .add_event::<FindConnectedNodesRequest>()
            .add_event::<FindConnectedNodesResponse>()
            .add_event::<ShortestPathRequest>()
            .add_event::<ShortestPathResponse>()
            .add_systems(
                bevy_app::Update,
                (
                    find_nodes_by_type_system,
                    find_connected_nodes_system,
                    calculate_shortest_path_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_nodes_by_type_request() {
        let request = FindNodesByTypeRequest {
            graph_id: GraphId::new(),
            node_type: NodeType::Data,
        };
        
        assert_eq!(request.node_type, NodeType::Data);
    }
    
    #[test]
    fn test_connected_nodes_response() {
        let mut connected = HashMap::new();
        connected.insert(NodeId::new(), 1);
        connected.insert(NodeId::new(), 2);
        
        let response = FindConnectedNodesResponse {
            graph_id: GraphId::new(),
            node_id: NodeId::new(),
            connected_nodes: connected.clone(),
        };
        
        assert_eq!(response.connected_nodes.len(), 2);
    }
} 