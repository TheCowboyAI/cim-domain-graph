//! Performance optimizations for large graphs
//!
//! This module provides optimized data structures and algorithms
//! for handling graphs with 10k+ nodes efficiently.

pub mod frustum_culling;
pub mod level_of_detail;
pub mod spatial_acceleration;
pub mod batched_renderer;
pub mod graph_partitioning;
pub mod incremental_layout;

use std::collections::HashMap;

/// Performance statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct GraphPerformanceStats {
    /// Total nodes in the graph
    pub total_nodes: usize,
    /// Nodes currently visible (after culling)
    pub visible_nodes: usize,
    /// Nodes rendered at each LOD level
    pub lod_distribution: HashMap<u8, usize>,
    /// Time spent on layout calculations (ms)
    pub layout_time_ms: f32,
    /// Time spent on rendering (ms)
    pub render_time_ms: f32,
    /// Memory usage (bytes)
    pub memory_usage: usize,
    /// Cache hit rate for queries
    pub query_cache_hit_rate: f32,
}

/// Configuration for performance optimizations
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable frustum culling
    pub frustum_culling: bool,
    /// Enable level of detail
    pub level_of_detail: bool,
    /// Enable spatial acceleration structures
    pub spatial_acceleration: bool,
    /// Enable batched rendering
    pub batched_rendering: bool,
    /// Maximum nodes to process per frame
    pub max_nodes_per_frame: usize,
    /// Distance thresholds for LOD levels
    pub lod_distances: Vec<f32>,
    /// Enable incremental layout updates
    pub incremental_layout: bool,
    /// Cache size for query results
    pub query_cache_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            frustum_culling: true,
            level_of_detail: true,
            spatial_acceleration: true,
            batched_rendering: true,
            max_nodes_per_frame: 5000,
            lod_distances: vec![100.0, 500.0, 1000.0, 2000.0],
            incremental_layout: true,
            query_cache_size: 1000,
        }
    }
}