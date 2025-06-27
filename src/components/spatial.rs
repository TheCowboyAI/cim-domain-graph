//! Spatial indexing ECS components

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::NodeId;

/// Spatial index for efficient queries
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SpatialIndex {
    pub index_type: IndexType,
    pub cell_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexType {
    /// Grid-based spatial index
    Grid,
    /// Quadtree index
    Quadtree,
    /// R-tree index
    RTree,
    /// KD-tree index
    KDTree,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self {
            index_type: IndexType::Grid,
            cell_size: 100.0,
        }
    }
}

/// Grid position for grid-based indexing
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridPosition {
    pub fn from_world_position(pos: &crate::value_objects::Position3D, cell_size: f32) -> Self {
        let cell_size_f64 = cell_size as f64;
        Self {
            x: (pos.x / cell_size_f64).floor() as i32,
            y: (pos.y / cell_size_f64).floor() as i32,
            z: (pos.z / cell_size_f64).floor() as i32,
        }
    }
}

/// Quadrant location for quadtree indexing
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuadrantLocation {
    pub level: u8,
    pub path: u64, // Bit-encoded path through quadtree
}

/// Proximity group for clustering nearby nodes
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ProximityGroup {
    pub group_id: u64,
    pub center: crate::value_objects::Position3D,
    pub radius: f32,
    pub members: Vec<NodeId>,
}

/// Spatial hash for fast lookups
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpatialHash {
    pub hash: u64,
}

impl SpatialHash {
    pub fn from_position(pos: &crate::value_objects::Position3D, precision: f32) -> Self {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        let precision_f64 = precision as f64;
        let x = (pos.x / precision_f64).round() as i64;
        let y = (pos.y / precision_f64).round() as i64;
        let z = (pos.z / precision_f64).round() as i64;
        
        x.hash(&mut hasher);
        y.hash(&mut hasher);
        z.hash(&mut hasher);
        
        Self {
            hash: hasher.finish(),
        }
    }
} 