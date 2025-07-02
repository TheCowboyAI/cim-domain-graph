//! Spatial indexing systems for efficient graph queries

use bevy_ecs::prelude::*;
use crate::components::{NodeEntity, GraphEntity};
use crate::value_objects::Position3D;
use crate::{NodeId, GraphId};
use rstar::{RTree, AABB, PointDistance, RTreeObject};
use std::collections::HashMap;

/// Spatial index entry for nodes
#[derive(Debug, Clone)]
struct SpatialNode {
    node_id: NodeId,
    position: [f64; 3],
}

impl RTreeObject for SpatialNode {
    type Envelope = AABB<[f64; 3]>;
    
    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.position)
    }
}

impl PointDistance for SpatialNode {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.position[0] - point[0];
        let dy = self.position[1] - point[1];
        let dz = self.position[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

/// Resource for spatial indices per graph
#[derive(Resource, Default)]
pub struct SpatialIndices {
    indices: HashMap<GraphId, RTree<SpatialNode>>,
}

/// Event to trigger spatial index update
#[derive(Event, Debug, Clone)]
pub struct UpdateSpatialIndexRequest {
    pub graph_id: GraphId,
}

/// Update spatial indices for efficient queries
pub fn update_spatial_index_system(
    mut spatial_indices: ResMut<SpatialIndices>,
    mut update_requests: EventReader<UpdateSpatialIndexRequest>,
    node_query: Query<(&NodeEntity, &Position3D, &Parent)>,
    graph_query: Query<&GraphEntity>,
) {
    for request in update_requests.read() {
        let mut nodes = Vec::new();
        
        // Collect all nodes for this graph
        for (node, position, parent) in &node_query {
            if let Ok(graph) = graph_query.get(parent.get()) {
                if graph.graph_id == request.graph_id {
                    nodes.push(SpatialNode {
                        node_id: node.node_id,
                        position: [position.x, position.y, position.z],
                    });
                }
            }
        }
        
        // Build or update the R-tree
        if !nodes.is_empty() {
            let rtree = RTree::bulk_load(nodes);
            spatial_indices.indices.insert(request.graph_id, rtree);
        }
    }
}

/// Event for region query requests
#[derive(Event, Debug, Clone)]
pub struct FindNodesInRegionRequest {
    pub graph_id: GraphId,
    pub min: Position3D,
    pub max: Position3D,
}

/// Event for region query responses
#[derive(Event, Debug, Clone)]
pub struct FindNodesInRegionResponse {
    pub graph_id: GraphId,
    pub nodes: Vec<(NodeId, Position3D)>,
}

/// Find nodes within a region (bounding box)
pub fn find_nodes_in_region_system(
    spatial_indices: Res<SpatialIndices>,
    mut region_requests: EventReader<FindNodesInRegionRequest>,
    mut region_responses: EventWriter<FindNodesInRegionResponse>,
) {
    for request in region_requests.read() {
        let mut nodes = Vec::new();
        
        if let Some(rtree) = spatial_indices.indices.get(&request.graph_id) {
            let envelope = AABB::from_corners(
                [request.min.x, request.min.y, request.min.z],
                [request.max.x, request.max.y, request.max.z],
            );
            
            for node in rtree.locate_in_envelope(&envelope) {
                nodes.push((
                    node.node_id,
                    Position3D {
                        x: node.position[0],
                        y: node.position[1],
                        z: node.position[2],
                    },
                ));
            }
        }
        
        region_responses.send(FindNodesInRegionResponse {
            graph_id: request.graph_id,
            nodes,
        });
    }
}

/// Event for nearest neighbor query
#[derive(Event, Debug, Clone)]
pub struct FindNearestNodesRequest {
    pub graph_id: GraphId,
    pub position: Position3D,
    pub count: usize,
}

/// Event for nearest neighbor response
#[derive(Event, Debug, Clone)]
pub struct FindNearestNodesResponse {
    pub graph_id: GraphId,
    pub nodes: Vec<(NodeId, Position3D, f64)>, // node_id, position, distance
}

/// Find nearest nodes to a position
pub fn find_nearest_nodes_system(
    spatial_indices: Res<SpatialIndices>,
    mut nearest_requests: EventReader<FindNearestNodesRequest>,
    mut nearest_responses: EventWriter<FindNearestNodesResponse>,
) {
    for request in nearest_requests.read() {
        let mut nodes = Vec::new();
        
        if let Some(rtree) = spatial_indices.indices.get(&request.graph_id) {
            let point = [request.position.x, request.position.y, request.position.z];
            
            for node in rtree.nearest_neighbor_iter(&point).take(request.count) {
                let distance = node.distance_2(&point).sqrt();
                nodes.push((
                    node.node_id,
                    Position3D {
                        x: node.position[0],
                        y: node.position[1],
                        z: node.position[2],
                    },
                    distance,
                ));
            }
        }
        
        nearest_responses.send(FindNearestNodesResponse {
            graph_id: request.graph_id,
            nodes,
        });
    }
}

/// Parameters for clustering
#[derive(Debug, Clone)]
pub struct ClusteringParams {
    pub min_cluster_size: usize,
    pub max_distance: f64,
}

/// Event for clustering request
#[derive(Event, Debug, Clone)]
pub struct ClusterNodesRequest {
    pub graph_id: GraphId,
    pub params: ClusteringParams,
}

/// Event for clustering response
#[derive(Event, Debug, Clone)]
pub struct ClusterNodesResponse {
    pub graph_id: GraphId,
    pub clusters: Vec<Vec<NodeId>>,
}

/// Cluster nearby nodes using DBSCAN-like algorithm
pub fn cluster_nearby_nodes_system(
    spatial_indices: Res<SpatialIndices>,
    mut cluster_requests: EventReader<ClusterNodesRequest>,
    mut cluster_responses: EventWriter<ClusterNodesResponse>,
) {
    for request in cluster_requests.read() {
        let mut clusters = Vec::new();
        
        if let Some(rtree) = spatial_indices.indices.get(&request.graph_id) {
            let all_nodes: Vec<_> = rtree.iter().cloned().collect();
            let mut visited = vec![false; all_nodes.len()];
            let mut cluster_assignments = vec![None; all_nodes.len()];
            
            for (i, node) in all_nodes.iter().enumerate() {
                if visited[i] {
                    continue;
                }
                
                visited[i] = true;
                
                // Find neighbors within distance
                let point = node.position;
                let neighbors: Vec<_> = rtree
                    .locate_within_distance(point, request.params.max_distance)
                    .enumerate()
                    .map(|(idx, _)| idx)
                    .collect();
                
                if neighbors.len() >= request.params.min_cluster_size {
                    // Start new cluster
                    let cluster_id = clusters.len();
                    let mut cluster = Vec::new();
                    let mut to_process = neighbors;
                    
                    while let Some(neighbor_idx) = to_process.pop() {
                        if !visited[neighbor_idx] {
                            visited[neighbor_idx] = true;
                            cluster_assignments[neighbor_idx] = Some(cluster_id);
                            cluster.push(all_nodes[neighbor_idx].node_id);
                            
                            // Find neighbors of this neighbor
                            let neighbor_point = all_nodes[neighbor_idx].position;
                            let new_neighbors: Vec<_> = rtree
                                .locate_within_distance(neighbor_point, request.params.max_distance)
                                .enumerate()
                                .filter(|(idx, _)| !visited[*idx])
                                .map(|(idx, _)| idx)
                                .collect();
                            
                            if new_neighbors.len() >= request.params.min_cluster_size {
                                to_process.extend(new_neighbors);
                            }
                        }
                    }
                    
                    if !cluster.is_empty() {
                        clusters.push(cluster);
                    }
                }
            }
        }
        
        cluster_responses.send(ClusterNodesResponse {
            graph_id: request.graph_id,
            clusters,
        });
    }
}

/// System to automatically update spatial index when nodes change position
pub fn auto_update_spatial_index(
    mut commands: Commands,
    changed_nodes: Query<(&Parent, &Position3D), Changed<Position3D>>,
    graph_query: Query<&GraphEntity>,
) {
    let mut graphs_to_update = std::collections::HashSet::new();
    
    for (parent, _) in &changed_nodes {
        if let Ok(graph) = graph_query.get(parent.get()) {
            graphs_to_update.insert(graph.graph_id);
        }
    }
    
    for graph_id in graphs_to_update {
        commands.add(|world: &mut World| {
            world.send_event(UpdateSpatialIndexRequest { graph_id });
        });
    }
}

/// Plugin to register spatial systems
pub struct SpatialSystemsPlugin;

impl bevy_app::Plugin for SpatialSystemsPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            .init_resource::<SpatialIndices>()
            .add_event::<UpdateSpatialIndexRequest>()
            .add_event::<FindNodesInRegionRequest>()
            .add_event::<FindNodesInRegionResponse>()
            .add_event::<FindNearestNodesRequest>()
            .add_event::<FindNearestNodesResponse>()
            .add_event::<ClusterNodesRequest>()
            .add_event::<ClusterNodesResponse>()
            .add_systems(
                bevy_app::Update,
                (
                    update_spatial_index_system,
                    find_nodes_in_region_system,
                    find_nearest_nodes_system,
                    cluster_nearby_nodes_system,
                    auto_update_spatial_index,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_node_distance() {
        let node = SpatialNode {
            node_id: NodeId::new(),
            position: [1.0, 2.0, 3.0],
        };
        
        let point = [4.0, 5.0, 6.0];
        let distance_squared = node.distance_2(&point);
        
        // Expected: (4-1)² + (5-2)² + (6-3)² = 9 + 9 + 9 = 27
        assert_eq!(distance_squared, 27.0);
    }
    
    #[test]
    fn test_clustering_params() {
        let params = ClusteringParams {
            min_cluster_size: 3,
            max_distance: 5.0,
        };
        
        assert_eq!(params.min_cluster_size, 3);
        assert_eq!(params.max_distance, 5.0);
    }
} 