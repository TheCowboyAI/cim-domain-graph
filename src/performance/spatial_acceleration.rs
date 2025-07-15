//! Spatial acceleration structures for efficient force-directed layouts
//!
//! Uses Barnes-Hut algorithm and spatial hashing to reduce force calculations
//! from O(n²) to O(n log n) for large graphs.

use std::collections::HashMap;
use crate::{NodeId, layout::advanced_layouts::Vec3};

/// Barnes-Hut quadtree/octree for 3D space
pub struct BarnesHutTree {
    root: Box<BHNode>,
    theta: f32,
}

/// Node in the Barnes-Hut tree
enum BHNode {
    /// Leaf node containing a single graph node
    Leaf {
        node_id: NodeId,
        position: Vec3,
        mass: f32,
    },
    /// Internal node containing multiple nodes
    Internal {
        center_of_mass: Vec3,
        total_mass: f32,
        bounds: Bounds3D,
        children: [Option<Box<BHNode>>; 8],
    },
}

/// 3D bounding box
#[derive(Debug, Clone)]
struct Bounds3D {
    min: Vec3,
    max: Vec3,
}

impl Bounds3D {
    fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    fn center(&self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    fn size(&self) -> f32 {
        let dx = self.max.x - self.min.x;
        let dy = self.max.y - self.min.y;
        let dz = self.max.z - self.min.z;
        dx.max(dy).max(dz)
    }

    fn octant(&self, position: &Vec3) -> usize {
        let center = self.center();
        let mut octant = 0;
        if position.x > center.x { octant |= 1; }
        if position.y > center.y { octant |= 2; }
        if position.z > center.z { octant |= 4; }
        octant
    }

    fn child_bounds(&self, octant: usize) -> Bounds3D {
        let center = self.center();
        let min = Vec3::new(
            if octant & 1 == 0 { self.min.x } else { center.x },
            if octant & 2 == 0 { self.min.y } else { center.y },
            if octant & 4 == 0 { self.min.z } else { center.z },
        );
        let max = Vec3::new(
            if octant & 1 == 0 { center.x } else { self.max.x },
            if octant & 2 == 0 { center.y } else { self.max.y },
            if octant & 4 == 0 { center.z } else { self.max.z },
        );
        Bounds3D::new(min, max)
    }
}

impl BarnesHutTree {
    /// Create a new Barnes-Hut tree
    pub fn new(nodes: &HashMap<NodeId, Vec3>, theta: f32) -> Self {
        if nodes.is_empty() {
            // Create dummy tree
            let bounds = Bounds3D::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
            let root = Box::new(BHNode::Internal {
                center_of_mass: Vec3::ZERO,
                total_mass: 0.0,
                bounds,
                children: Default::default(),
            });
            return Self { root, theta };
        }

        // Calculate bounds
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for pos in nodes.values() {
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }

        // Add padding to bounds
        let padding = (max - min).length() * 0.01;
        min = min - Vec3::new(padding, padding, padding);
        max = max + Vec3::new(padding, padding, padding);

        let bounds = Bounds3D::new(min, max);
        let mut root = Box::new(BHNode::Internal {
            center_of_mass: Vec3::ZERO,
            total_mass: 0.0,
            bounds,
            children: Default::default(),
        });

        // Insert all nodes
        for (node_id, position) in nodes {
            Self::insert(&mut root, node_id.clone(), *position, 1.0);
        }

        // Calculate centers of mass
        Self::calculate_centers(&mut root);

        Self { root, theta }
    }

    /// Insert a node into the tree
    fn insert(node: &mut Box<BHNode>, node_id: NodeId, position: Vec3, mass: f32) {
        match node.as_mut() {
            BHNode::Internal { bounds, children, .. } => {
                let octant = bounds.octant(&position);
                let _child_bounds = bounds.child_bounds(octant);

                if let Some(child) = &mut children[octant] {
                    Self::insert(child, node_id, position, mass);
                } else {
                    children[octant] = Some(Box::new(BHNode::Leaf {
                        node_id,
                        position,
                        mass,
                    }));
                }
            }
            BHNode::Leaf { .. } => {
                // Convert leaf to internal node
                let old_leaf = std::mem::replace(
                    node.as_mut(),
                    BHNode::Internal {
                        center_of_mass: Vec3::ZERO,
                        total_mass: 0.0,
                        bounds: Bounds3D::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0)),
                        children: Default::default(),
                    }
                );

                if let (
                    BHNode::Leaf { node_id: old_id, position: old_pos, mass: old_mass },
                    BHNode::Internal { bounds, children, .. }
                ) = (old_leaf, node.as_mut()) {
                    // Reinsert old node
                    let octant = bounds.octant(&old_pos);
                    children[octant] = Some(Box::new(BHNode::Leaf {
                        node_id: old_id,
                        position: old_pos,
                        mass: old_mass,
                    }));

                    // Insert new node
                    Self::insert(node, node_id, position, mass);
                }
            }
        }
    }

    /// Calculate centers of mass for all internal nodes
    fn calculate_centers(node: &mut Box<BHNode>) -> (Vec3, f32) {
        match node.as_mut() {
            BHNode::Leaf { position, mass, .. } => (*position, *mass),
            BHNode::Internal { center_of_mass, total_mass, children, .. } => {
                let mut com = Vec3::ZERO;
                let mut mass_sum = 0.0;

                for child in children.iter_mut().flatten() {
                    let (child_com, child_mass) = Self::calculate_centers(child);
                    com = com + child_com * child_mass;
                    mass_sum += child_mass;
                }

                if mass_sum > 0.0 {
                    com = com * (1.0 / mass_sum);
                }

                *center_of_mass = com;
                *total_mass = mass_sum;
                (com, mass_sum)
            }
        }
    }

    /// Calculate force on a node using Barnes-Hut approximation
    pub fn calculate_force(&self, node_id: &NodeId, position: Vec3, k_squared: f32) -> Vec3 {
        Self::calculate_force_recursive(&self.root, node_id, position, k_squared, self.theta)
    }

    fn calculate_force_recursive(
        node: &Box<BHNode>,
        target_id: &NodeId,
        target_pos: Vec3,
        k_squared: f32,
        theta: f32,
    ) -> Vec3 {
        match node.as_ref() {
            BHNode::Leaf { node_id, position, .. } => {
                if node_id == target_id {
                    Vec3::ZERO
                } else {
                    let delta = target_pos - *position;
                    let distance = delta.length().max(0.01);
                    let repulsive_force = k_squared / (distance * distance);
                    delta.normalize() * repulsive_force
                }
            }
            BHNode::Internal { center_of_mass, total_mass, bounds, children } => {
                let delta = target_pos - *center_of_mass;
                let distance = delta.length();

                if distance > 0.0 {
                    let size = bounds.size();
                    let ratio = size / distance;

                    if ratio < theta {
                        // Treat as single body
                        let repulsive_force = k_squared * *total_mass / (distance * distance);
                        return delta.normalize() * repulsive_force;
                    }
                }

                // Recurse into children
                let mut force = Vec3::ZERO;
                for child in children.iter().flatten() {
                    force = force + Self::calculate_force_recursive(child, target_id, target_pos, k_squared, theta);
                }
                force
            }
        }
    }
}

/// Spatial hash grid for neighbor queries
pub struct SpatialHashGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<NodeId>>,
}

impl SpatialHashGrid {
    /// Create a new spatial hash grid
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Build the grid from node positions
    pub fn build(&mut self, nodes: &HashMap<NodeId, Vec3>) {
        self.cells.clear();
        
        for (node_id, position) in nodes {
            let cell = self.position_to_cell(position);
            self.cells.entry(cell).or_default().push(node_id.clone());
        }
    }

    /// Get the cell coordinates for a position
    fn position_to_cell(&self, position: &Vec3) -> (i32, i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }

    /// Find all nodes within a radius of a position
    pub fn find_neighbors(&self, position: &Vec3, radius: f32) -> Vec<NodeId> {
        let mut neighbors = Vec::new();
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.position_to_cell(position);

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    let cell = (
                        center_cell.0 + dx,
                        center_cell.1 + dy,
                        center_cell.2 + dz,
                    );
                    
                    if let Some(nodes) = self.cells.get(&cell) {
                        neighbors.extend(nodes.clone());
                    }
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barnes_hut_tree() {
        let mut nodes = HashMap::new();
        nodes.insert(NodeId::new(), Vec3::new(0.0, 0.0, 0.0));
        nodes.insert(NodeId::new(), Vec3::new(100.0, 0.0, 0.0));
        nodes.insert(NodeId::new(), Vec3::new(0.0, 100.0, 0.0));
        nodes.insert(NodeId::new(), Vec3::new(0.0, 0.0, 100.0));

        let tree = BarnesHutTree::new(&nodes, 0.5);
        
        // Test force calculation
        let test_pos = Vec3::new(50.0, 50.0, 50.0);
        let force = tree.calculate_force(&NodeId::new(), test_pos, 100.0);
        
        // Force should be non-zero
        assert!(force.length() > 0.0);
    }

    #[test]
    fn test_spatial_hash_grid() {
        let mut grid = SpatialHashGrid::new(50.0);
        let mut nodes = HashMap::new();
        
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        let id3 = NodeId::new();
        
        nodes.insert(id1.clone(), Vec3::new(0.0, 0.0, 0.0));
        nodes.insert(id2.clone(), Vec3::new(10.0, 10.0, 0.0));
        nodes.insert(id3.clone(), Vec3::new(100.0, 100.0, 0.0));
        
        grid.build(&nodes);
        
        // Find neighbors near origin
        let neighbors = grid.find_neighbors(&Vec3::new(5.0, 5.0, 0.0), 20.0);
        assert!(neighbors.contains(&id1));
        assert!(neighbors.contains(&id2));
        assert!(!neighbors.contains(&id3));
    }
}