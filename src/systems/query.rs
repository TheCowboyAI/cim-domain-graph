//! Graph query systems

use crate::{
    components::{EdgeEntity, EdgeWeight, NodeEntity, NodeMetadata, NodeType},
    value_objects::Position3D,
    GraphId, NodeId,
};
use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};

/// Query request for finding nodes by criteria
#[derive(Event)]
pub struct FindNodesRequest {
    pub graph_id: Option<GraphId>,
    pub node_type: Option<String>,
    pub tags: Vec<String>,
    pub metadata_filter: HashMap<String, serde_json::Value>,
}

/// Query response containing found nodes
#[derive(Event)]
pub struct FindNodesResponse {
    pub request_id: uuid::Uuid,
    pub nodes: Vec<NodeInfo>,
}

/// Node information for query responses
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub position: Position3D,
    pub node_type: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System that finds nodes matching criteria
pub fn find_nodes_system(
    mut requests: EventReader<FindNodesRequest>,
    mut responses: EventWriter<FindNodesResponse>,
    node_query: Query<(
        &NodeEntity,
        &Position3D,
        Option<&NodeType>,
        Option<&NodeMetadata>,
    )>,
) {
    for request in requests.read() {
        let mut found_nodes = Vec::new();

        for (entity, position, node_type, metadata) in node_query.iter() {
            // Filter by graph if specified
            if let Some(graph_id) = request.graph_id {
                if entity.graph_id != graph_id {
                    continue;
                }
            }

            // Filter by node type if specified
            if let Some(requested_type) = &request.node_type {
                let type_matches = node_type
                    .map(|t| format!("{:?}", t).to_lowercase() == requested_type.to_lowercase())
                    .unwrap_or(false);

                if !type_matches {
                    continue;
                }
            }

            // Filter by tags if specified
            if !request.tags.is_empty() {
                let has_all_tags = metadata
                    .map(|m| request.tags.iter().all(|tag| m.tags.contains(tag)))
                    .unwrap_or(false);

                if !has_all_tags {
                    continue;
                }
            }

            // Filter by metadata if specified
            if !request.metadata_filter.is_empty() {
                let matches_filter = metadata
                    .map(|m| {
                        request
                            .metadata_filter
                            .iter()
                            .all(|(key, value)| m.properties.get(key) == Some(value))
                    })
                    .unwrap_or(false);

                if !matches_filter {
                    continue;
                }
            }

            // Node matches all criteria
            found_nodes.push(NodeInfo {
                node_id: entity.node_id,
                graph_id: entity.graph_id,
                position: position.clone(),
                node_type: node_type.map(|t| format!("{:?}", t)),
                metadata: metadata.map(|m| m.properties.clone()).unwrap_or_default(),
            });
        }

        responses.write(FindNodesResponse {
            request_id: uuid::Uuid::new_v4(),
            nodes: found_nodes,
        });
    }
}

/// Query request for finding shortest path
#[derive(Event)]
pub struct FindShortestPathRequest {
    pub graph_id: GraphId,
    pub source: NodeId,
    pub target: NodeId,
}

/// Query response containing path
#[derive(Event)]
pub struct FindShortestPathResponse {
    pub request_id: uuid::Uuid,
    pub path: Option<Vec<NodeId>>,
    pub total_distance: f32,
}

/// System that finds shortest path between nodes
pub fn find_shortest_path_system(
    mut requests: EventReader<FindShortestPathRequest>,
    mut responses: EventWriter<FindShortestPathResponse>,
    _node_query: Query<(&NodeEntity, &Position3D)>,
    edge_query: Query<(&EdgeEntity, Option<&EdgeWeight>)>,
) {
    for request in requests.read() {
        // Build adjacency list for the graph
        let mut adjacency: HashMap<NodeId, Vec<(NodeId, f32)>> = HashMap::new();

        for (edge, weight) in edge_query.iter() {
            if edge.graph_id == request.graph_id {
                let edge_weight = weight.map(|w| w.0).unwrap_or(1.0);

                adjacency
                    .entry(edge.source)
                    .or_default()
                    .push((edge.target, edge_weight));

                // For undirected edges, add reverse connection
                // (In a real implementation, check EdgeType)
            }
        }

        // Use Dijkstra's algorithm
        let path = dijkstra(&adjacency, request.source, request.target);

        // Calculate total distance
        let total_distance = if let Some(ref path) = path {
            let mut distance = 0.0;
            for i in 0..path.len() - 1 {
                if let Some(neighbors) = adjacency.get(&path[i]) {
                    if let Some((_, weight)) = neighbors.iter().find(|(n, _)| *n == path[i + 1]) {
                        distance += weight;
                    }
                }
            }
            distance
        } else {
            f32::INFINITY
        };

        responses.write(FindShortestPathResponse {
            request_id: uuid::Uuid::new_v4(),
            path,
            total_distance,
        });
    }
}

/// Query request for finding connected components
#[derive(Event)]
pub struct FindConnectedComponentsRequest {
    pub graph_id: GraphId,
}

/// Query response containing components
#[derive(Event)]
pub struct FindConnectedComponentsResponse {
    pub request_id: uuid::Uuid,
    pub components: Vec<Vec<NodeId>>,
}

/// System that finds connected components in a graph
pub fn find_connected_components_system(
    mut requests: EventReader<FindConnectedComponentsRequest>,
    mut responses: EventWriter<FindConnectedComponentsResponse>,
    node_query: Query<&NodeEntity>,
    edge_query: Query<&EdgeEntity>,
) {
    for request in requests.read() {
        // Collect all nodes in the graph
        let nodes: Vec<NodeId> = node_query
            .iter()
            .filter(|n| n.graph_id == request.graph_id)
            .map(|n| n.node_id)
            .collect();

        // Build adjacency list
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for node in &nodes {
            adjacency.insert(*node, Vec::new());
        }

        for edge in edge_query.iter() {
            if edge.graph_id == request.graph_id {
                adjacency.get_mut(&edge.source).unwrap().push(edge.target);
                adjacency.get_mut(&edge.target).unwrap().push(edge.source);
            }
        }

        // Find components using DFS
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node in nodes {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                let mut stack = vec![node];

                while let Some(current) = stack.pop() {
                    if visited.insert(current) {
                        component.push(current);

                        if let Some(neighbors) = adjacency.get(&current) {
                            for neighbor in neighbors {
                                if !visited.contains(neighbor) {
                                    stack.push(*neighbor);
                                }
                            }
                        }
                    }
                }

                if !component.is_empty() {
                    components.push(component);
                }
            }
        }

        responses.write(FindConnectedComponentsResponse {
            request_id: uuid::Uuid::new_v4(),
            components,
        });
    }
}

/// Helper function for Dijkstra's algorithm
fn dijkstra(
    adjacency: &HashMap<NodeId, Vec<(NodeId, f32)>>,
    source: NodeId,
    target: NodeId,
) -> Option<Vec<NodeId>> {
    use std::cmp::Ordering;

    #[derive(PartialEq)]
    struct State {
        cost: f32,
        node: NodeId,
    }

    impl Eq for State {}

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            other.cost.partial_cmp(&self.cost)
        }
    }

    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    let mut distances: HashMap<NodeId, f32> = HashMap::new();
    let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
    let mut heap = std::collections::BinaryHeap::new();

    distances.insert(source, 0.0);
    heap.push(State {
        cost: 0.0,
        node: source,
    });

    while let Some(State { cost, node }) = heap.pop() {
        if node == target {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = target;

            while current != source {
                path.push(current);
                current = *previous.get(&current)?;
            }
            path.push(source);
            path.reverse();

            return Some(path);
        }

        if cost > *distances.get(&node).unwrap_or(&f32::INFINITY) {
            continue;
        }

        if let Some(neighbors) = adjacency.get(&node) {
            for (neighbor, weight) in neighbors {
                let next_cost = cost + weight;

                if next_cost < *distances.get(neighbor).unwrap_or(&f32::INFINITY) {
                    distances.insert(*neighbor, next_cost);
                    previous.insert(*neighbor, node);
                    heap.push(State {
                        cost: next_cost,
                        node: *neighbor,
                    });
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<FindNodesRequest>>();
        world.init_resource::<Events<FindNodesResponse>>();
        world.init_resource::<Events<FindShortestPathRequest>>();
        world.init_resource::<Events<FindShortestPathResponse>>();
        world.init_resource::<Events<FindConnectedComponentsRequest>>();
        world.init_resource::<Events<FindConnectedComponentsResponse>>();
        world
    }

    #[test]
    fn test_find_nodes_by_type() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();

        // Create nodes with different types
        world.spawn((
            NodeEntity {
                node_id: NodeId::new(),
                graph_id,
            },
            Position3D::default(),
            NodeType::Start,
        ));

        world.spawn((
            NodeEntity {
                node_id: NodeId::new(),
                graph_id,
            },
            Position3D::default(),
            NodeType::Process,
        ));

        // Send query request
        world
            .resource_mut::<Events<FindNodesRequest>>()
            .send(FindNodesRequest {
                graph_id: Some(graph_id),
                node_type: Some("Start".to_string()),
                tags: vec![],
                metadata_filter: HashMap::new(),
            });

        // Run the system
        let mut find_system = IntoSystem::into_system(find_nodes_system);
        find_system.initialize(&mut world);
        find_system.run((), &mut world);
        find_system.apply_deferred(&mut world);

        // Check response
        let mut response_reader = world.resource_mut::<Events<FindNodesResponse>>();
        let responses: Vec<_> = response_reader.drain().collect();

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].nodes.len(), 1);
        assert_eq!(responses[0].nodes[0].node_type, Some("Start".to_string()));
    }

    #[test]
    fn test_shortest_path() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();

        // Create a simple graph: A -> B -> C
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let node_c = NodeId::new();

        world.spawn((
            NodeEntity {
                node_id: node_a,
                graph_id,
            },
            Position3D::default(),
        ));

        world.spawn((
            NodeEntity {
                node_id: node_b,
                graph_id,
            },
            Position3D::default(),
        ));

        world.spawn((
            NodeEntity {
                node_id: node_c,
                graph_id,
            },
            Position3D::default(),
        ));

        // Add edges
        world.spawn((
            EdgeEntity {
                edge_id: crate::EdgeId::new(),
                source: node_a,
                target: node_b,
                graph_id,
            },
            EdgeWeight(1.0),
        ));

        world.spawn((
            EdgeEntity {
                edge_id: crate::EdgeId::new(),
                source: node_b,
                target: node_c,
                graph_id,
            },
            EdgeWeight(1.0),
        ));

        // Send path request
        world
            .resource_mut::<Events<FindShortestPathRequest>>()
            .send(FindShortestPathRequest {
                graph_id,
                source: node_a,
                target: node_c,
            });

        // Run the system
        let mut path_system = IntoSystem::into_system(find_shortest_path_system);
        path_system.initialize(&mut world);
        path_system.run((), &mut world);
        path_system.apply_deferred(&mut world);

        // Check response
        let mut response_reader = world.resource_mut::<Events<FindShortestPathResponse>>();
        let responses: Vec<_> = response_reader.drain().collect();

        assert_eq!(responses.len(), 1);
        assert!(responses[0].path.is_some());
        let path = responses[0].path.as_ref().unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], node_a);
        assert_eq!(path[1], node_b);
        assert_eq!(path[2], node_c);
        assert_eq!(responses[0].total_distance, 2.0);
    }
}
