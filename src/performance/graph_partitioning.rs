//! Graph partitioning for improved cache locality and parallel processing
//!
//! Divides large graphs into smaller partitions that can be processed
//! independently, improving memory access patterns and enabling parallelism.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::{NodeId, EdgeId};

/// A partition of the graph
#[derive(Debug, Clone)]
pub struct GraphPartition {
    /// Unique identifier for this partition
    pub id: usize,
    /// Nodes in this partition
    pub nodes: HashSet<NodeId>,
    /// Internal edges (both endpoints in this partition)
    pub internal_edges: HashSet<EdgeId>,
    /// Cut edges (one endpoint in another partition)
    pub cut_edges: HashSet<EdgeId>,
    /// Neighboring partitions
    pub neighbors: HashSet<usize>,
}

impl GraphPartition {
    /// Create a new empty partition
    pub fn new(id: usize) -> Self {
        Self {
            id,
            nodes: HashSet::new(),
            internal_edges: HashSet::new(),
            cut_edges: HashSet::new(),
            neighbors: HashSet::new(),
        }
    }

    /// Get the size of this partition
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    /// Get the edge cut size (number of edges to other partitions)
    pub fn edge_cut(&self) -> usize {
        self.cut_edges.len()
    }

    /// Calculate the modularity contribution of this partition
    pub fn modularity(&self) -> f32 {
        let internal = self.internal_edges.len() as f32;
        let cut = self.cut_edges.len() as f32;
        let total = internal + cut;
        
        if total > 0.0 {
            internal / total
        } else {
            0.0
        }
    }
}

/// Graph partitioning result
#[derive(Debug)]
pub struct PartitioningResult {
    /// All partitions
    pub partitions: Vec<GraphPartition>,
    /// Node to partition mapping
    pub node_partition: HashMap<NodeId, usize>,
    /// Quality metrics
    pub metrics: PartitioningMetrics,
}

#[derive(Debug, Clone)]
pub struct PartitioningMetrics {
    /// Number of partitions
    pub partition_count: usize,
    /// Average partition size
    pub avg_partition_size: f32,
    /// Standard deviation of partition sizes
    pub size_std_dev: f32,
    /// Total edge cut (edges between partitions)
    pub total_edge_cut: usize,
    /// Modularity score (0-1, higher is better)
    pub modularity: f32,
    /// Load balance factor (0-1, 1 is perfect balance)
    pub balance_factor: f32,
}

/// Configuration for graph partitioning
#[derive(Debug, Clone)]
pub struct PartitioningConfig {
    /// Target number of partitions (0 for automatic)
    pub target_partitions: usize,
    /// Maximum nodes per partition
    pub max_partition_size: usize,
    /// Minimum nodes per partition
    pub min_partition_size: usize,
    /// Algorithm to use
    pub algorithm: PartitioningAlgorithm,
    /// Whether to minimize edge cut
    pub minimize_edge_cut: bool,
    /// Balance factor importance (0-1)
    pub balance_weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartitioningAlgorithm {
    /// METIS-style multilevel partitioning
    Multilevel,
    /// Simple BFS-based partitioning
    BreadthFirst,
    /// Spectral partitioning
    Spectral,
    /// Community detection (Louvain)
    Community,
}

impl Default for PartitioningConfig {
    fn default() -> Self {
        Self {
            target_partitions: 0, // Automatic
            max_partition_size: 1000,
            min_partition_size: 10,
            algorithm: PartitioningAlgorithm::Multilevel,
            minimize_edge_cut: true,
            balance_weight: 0.5,
        }
    }
}

/// Graph partitioner
pub struct GraphPartitioner {
    config: PartitioningConfig,
}

impl GraphPartitioner {
    /// Create a new partitioner
    pub fn new(config: PartitioningConfig) -> Self {
        Self { config }
    }

    /// Partition a graph
    pub fn partition(
        &self,
        nodes: &[NodeId],
        edges: &[(NodeId, NodeId, EdgeId)],
    ) -> PartitioningResult {
        match self.config.algorithm {
            PartitioningAlgorithm::BreadthFirst => self.bfs_partition(nodes, edges),
            PartitioningAlgorithm::Multilevel => self.multilevel_partition(nodes, edges),
            PartitioningAlgorithm::Community => self.community_partition(nodes, edges),
            PartitioningAlgorithm::Spectral => self.spectral_partition(nodes, edges),
        }
    }

    /// Simple BFS-based partitioning
    fn bfs_partition(
        &self,
        nodes: &[NodeId],
        edges: &[(NodeId, NodeId, EdgeId)],
    ) -> PartitioningResult {
        // Build adjacency list
        let mut adjacency: HashMap<NodeId, Vec<(NodeId, EdgeId)>> = HashMap::new();
        for (source, target, edge_id) in edges {
            adjacency.entry(source.clone()).or_default().push((target.clone(), edge_id.clone()));
            adjacency.entry(target.clone()).or_default().push((source.clone(), edge_id.clone()));
        }

        // Calculate target partition count
        let target_count = if self.config.target_partitions > 0 {
            self.config.target_partitions
        } else {
            (nodes.len() / self.config.max_partition_size).max(1)
        };

        let target_size = nodes.len() / target_count;
        
        let mut partitions = Vec::new();
        let mut node_partition = HashMap::new();
        let mut unvisited: HashSet<_> = nodes.iter().cloned().collect();

        let mut partition_id = 0;

        while !unvisited.is_empty() {
            // Start new partition from random unvisited node
            let start_node = unvisited.iter().next().unwrap().clone();
            let mut partition = GraphPartition::new(partition_id);
            let mut queue = VecDeque::new();
            queue.push_back(start_node);

            // BFS to grow partition
            while let Some(node) = queue.pop_front() {
                if !unvisited.contains(&node) {
                    continue;
                }

                if partition.size() >= target_size && partition.size() >= self.config.min_partition_size {
                    break;
                }

                unvisited.remove(&node);
                partition.nodes.insert(node.clone());
                node_partition.insert(node.clone(), partition_id);

                // Add neighbors to queue
                if let Some(neighbors) = adjacency.get(&node) {
                    for (neighbor, _) in neighbors {
                        if unvisited.contains(neighbor) {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }

            partitions.push(partition);
            partition_id += 1;
        }

        // Classify edges
        for (source, target, edge_id) in edges {
            let source_partition = node_partition.get(source);
            let target_partition = node_partition.get(target);

            match (source_partition, target_partition) {
                (Some(&p1), Some(&p2)) if p1 == p2 => {
                    partitions[p1].internal_edges.insert(edge_id.clone());
                }
                (Some(&p1), Some(&p2)) => {
                    partitions[p1].cut_edges.insert(edge_id.clone());
                    partitions[p1].neighbors.insert(p2);
                    partitions[p2].neighbors.insert(p1);
                }
                _ => {}
            }
        }

        let metrics = self.calculate_metrics(&partitions);

        PartitioningResult {
            partitions,
            node_partition,
            metrics,
        }
    }

    /// Multilevel partitioning (simplified version)
    fn multilevel_partition(
        &self,
        nodes: &[NodeId],
        edges: &[(NodeId, NodeId, EdgeId)],
    ) -> PartitioningResult {
        // For now, fall back to BFS
        // Full implementation would include:
        // 1. Coarsening phase (edge contraction)
        // 2. Initial partitioning of coarse graph
        // 3. Uncoarsening with refinement
        self.bfs_partition(nodes, edges)
    }

    /// Community detection partitioning
    fn community_partition(
        &self,
        nodes: &[NodeId],
        edges: &[(NodeId, NodeId, EdgeId)],
    ) -> PartitioningResult {
        // Simplified Louvain algorithm
        // Full implementation would include modularity optimization
        self.bfs_partition(nodes, edges)
    }

    /// Spectral partitioning
    fn spectral_partition(
        &self,
        nodes: &[NodeId],
        edges: &[(NodeId, NodeId, EdgeId)],
    ) -> PartitioningResult {
        // Would require eigenvalue computation
        // For now, fall back to BFS
        self.bfs_partition(nodes, edges)
    }

    /// Calculate partitioning quality metrics
    fn calculate_metrics(&self, partitions: &[GraphPartition]) -> PartitioningMetrics {
        let partition_count = partitions.len();
        
        // Calculate sizes
        let sizes: Vec<f32> = partitions.iter().map(|p| p.size() as f32).collect();
        let avg_size = sizes.iter().sum::<f32>() / partition_count as f32;
        
        // Calculate standard deviation
        let variance = sizes.iter()
            .map(|&size| (size - avg_size).powi(2))
            .sum::<f32>() / partition_count as f32;
        let std_dev = variance.sqrt();

        // Calculate total edge cut
        let total_edge_cut = partitions.iter()
            .map(|p| p.edge_cut())
            .sum::<usize>() / 2; // Each cut edge counted twice

        // Calculate modularity
        let total_internal_edges: usize = partitions.iter()
            .map(|p| p.internal_edges.len())
            .sum();
        let total_edges = total_internal_edges + total_edge_cut;
        let modularity = if total_edges > 0 {
            total_internal_edges as f32 / total_edges as f32
        } else {
            0.0
        };

        // Calculate balance factor
        let max_size = sizes.iter().fold(0.0f32, |a, &b| a.max(b));
        let min_size = sizes.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let balance_factor = if max_size > 0.0 {
            min_size / max_size
        } else {
            1.0
        };

        PartitioningMetrics {
            partition_count,
            avg_partition_size: avg_size,
            size_std_dev: std_dev,
            total_edge_cut,
            modularity,
            balance_factor,
        }
    }
}

/// Hierarchical graph representation for multi-level processing
pub struct HierarchicalGraph {
    /// Levels of the hierarchy (0 is original, higher is coarser)
    pub levels: Vec<GraphLevel>,
    /// Current active level
    pub current_level: usize,
}

pub struct GraphLevel {
    /// Nodes at this level
    pub nodes: Vec<NodeId>,
    /// Edges at this level
    pub edges: Vec<(NodeId, NodeId, EdgeId)>,
    /// Mapping from this level to finer level
    pub refinement_map: HashMap<NodeId, Vec<NodeId>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_partitioning() {
        let nodes: Vec<NodeId> = (0..10).map(|_| NodeId::new()).collect();
        let edges: Vec<(NodeId, NodeId, EdgeId)> = vec![
            (nodes[0].clone(), nodes[1].clone(), EdgeId::new()),
            (nodes[1].clone(), nodes[2].clone(), EdgeId::new()),
            (nodes[2].clone(), nodes[3].clone(), EdgeId::new()),
            (nodes[3].clone(), nodes[4].clone(), EdgeId::new()),
            (nodes[5].clone(), nodes[6].clone(), EdgeId::new()),
            (nodes[6].clone(), nodes[7].clone(), EdgeId::new()),
            (nodes[7].clone(), nodes[8].clone(), EdgeId::new()),
            (nodes[8].clone(), nodes[9].clone(), EdgeId::new()),
            // Cut edge
            (nodes[4].clone(), nodes[5].clone(), EdgeId::new()),
        ];

        let config = PartitioningConfig {
            target_partitions: 2,
            max_partition_size: 5,
            min_partition_size: 2,
            ..Default::default()
        };

        let partitioner = GraphPartitioner::new(config);
        let result = partitioner.partition(&nodes, &edges);

        assert_eq!(result.partitions.len(), 2);
        assert!(result.metrics.total_edge_cut >= 1);
        assert!(result.metrics.balance_factor > 0.0);
    }
}