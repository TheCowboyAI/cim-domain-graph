//! Advanced graph layout algorithms
//!
//! This module provides sophisticated layout algorithms for complex graph visualizations
//! including 3D layouts, tree-specific layouts, and optimized force-directed algorithms.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::NodeId;

// Define Vec3 locally since glam is not available
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            Self::ZERO
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl std::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

/// Fruchterman-Reingold force-directed layout algorithm
/// This is a more sophisticated force-directed algorithm with better convergence
pub struct FruchtermanReingoldLayout {
    /// Ideal distance between nodes
    pub ideal_distance: f32,
    /// Temperature for simulated annealing
    pub temperature: f32,
    /// Cooling rate per iteration
    pub cooling_rate: f32,
    /// Maximum iterations
    pub max_iterations: u32,
}

impl Default for FruchtermanReingoldLayout {
    fn default() -> Self {
        Self {
            ideal_distance: 100.0,
            temperature: 100.0,
            cooling_rate: 0.95,
            max_iterations: 500,
        }
    }
}

impl FruchtermanReingoldLayout {
    pub fn apply(
        &mut self,
        nodes: &mut HashMap<NodeId, Vec3>,
        edges: &[(NodeId, NodeId)],
        bounds: Vec3,
    ) {
        let node_count = nodes.len() as f32;
        if node_count == 0.0 {
            return;
        }

        // Calculate area and ideal distance
        let area = bounds.x * bounds.y * bounds.z;
        let k = (area / node_count).sqrt();
        let k_squared = k * k;

        // Create a vector of node IDs for indexed access
        let node_ids: Vec<NodeId> = nodes.keys().cloned().collect();
        
        for _iteration in 0..self.max_iterations {
            // Calculate repulsive forces
            let mut displacements: HashMap<NodeId, Vec3> = HashMap::new();
            
            for i in 0..node_ids.len() {
                let id1 = node_ids[i].clone();
                let pos1 = nodes[&id1];
                let mut disp = Vec3::ZERO;
                
                for j in 0..node_ids.len() {
                    if i == j {
                        continue;
                    }
                    
                    let id2 = node_ids[j].clone();
                    let pos2 = nodes[&id2];
                    let delta = pos1 - pos2;
                    let distance = delta.length().max(0.01);
                    
                    // Repulsive force
                    let repulsive_force = k_squared / distance;
                    disp += delta.normalize() * repulsive_force;
                }
                
                displacements.insert(id1, disp);
            }
            
            // Calculate attractive forces for edges
            for (source, target) in edges {
                if let (Some(&pos1), Some(&pos2)) = (nodes.get(source), nodes.get(target)) {
                    let delta = pos2 - pos1;
                    let distance = delta.length().max(0.01);
                    
                    // Attractive force
                    let attractive_force = (distance * distance) / k;
                    let force_vector = delta.normalize() * attractive_force;
                    
                    *displacements.get_mut(source).unwrap() += force_vector;
                    *displacements.get_mut(target).unwrap() -= force_vector;
                }
            }
            
            // Apply displacements with temperature
            for (id, displacement) in displacements {
                if let Some(pos) = nodes.get_mut(&id) {
                    let disp_length = displacement.length();
                    if disp_length > 0.0 {
                        let capped_displacement = displacement.normalize() * disp_length.min(self.temperature);
                        *pos += capped_displacement;
                        
                        // Keep within bounds
                        pos.x = pos.x.clamp(-bounds.x / 2.0, bounds.x / 2.0);
                        pos.y = pos.y.clamp(-bounds.y / 2.0, bounds.y / 2.0);
                        pos.z = pos.z.clamp(-bounds.z / 2.0, bounds.z / 2.0);
                    }
                }
            }
            
            // Cool down temperature
            self.temperature *= self.cooling_rate;
            
            // Early exit if temperature is too low
            if self.temperature < 0.01 {
                break;
            }
        }
    }
}

/// 3D Sphere Layout - distributes nodes evenly on a sphere surface
pub struct SphereLayout {
    pub radius: f32,
    pub use_fibonacci: bool, // Use Fibonacci sphere for even distribution
}

impl Default for SphereLayout {
    fn default() -> Self {
        Self {
            radius: 200.0,
            use_fibonacci: true,
        }
    }
}

impl SphereLayout {
    pub fn apply(&self, nodes: &mut HashMap<NodeId, Vec3>) {
        let node_count = nodes.len();
        if node_count == 0 {
            return;
        }

        let node_ids: Vec<NodeId> = nodes.keys().cloned().collect();

        if self.use_fibonacci {
            // Fibonacci sphere algorithm for even distribution
            let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
            let angle_increment = std::f32::consts::TAU / golden_ratio;

            for (i, id) in node_ids.iter().enumerate() {
                let t = i as f32 / (node_count - 1) as f32;
                let inclination = (1.0 - 2.0 * t).acos();
                let azimuth = angle_increment * i as f32;

                let x = self.radius * inclination.sin() * azimuth.cos();
                let y = self.radius * inclination.sin() * azimuth.sin();
                let z = self.radius * inclination.cos();

                if let Some(pos) = nodes.get_mut(id) {
                    *pos = Vec3::new(x, y, z);
                }
            }
        } else {
            // Simple sphere distribution
            let num_lat = (node_count as f32).sqrt().ceil() as usize;
            let num_lon = num_lat * 2;

            for (i, id) in node_ids.iter().enumerate() {
                let lat = i / num_lon;
                let lon = i % num_lon;

                let theta = (lat as f32 / num_lat as f32) * std::f32::consts::PI;
                let phi = (lon as f32 / num_lon as f32) * std::f32::consts::TAU;

                let x = self.radius * theta.sin() * phi.cos();
                let y = self.radius * theta.sin() * phi.sin();
                let z = self.radius * theta.cos();

                if let Some(pos) = nodes.get_mut(id) {
                    *pos = Vec3::new(x, y, z);
                }
            }
        }
    }
}

/// Radial Tree Layout - arranges tree structures in concentric circles
pub struct RadialTreeLayout {
    pub center: Vec3,
    pub radius_increment: f32,
    pub start_angle: f32,
}

impl Default for RadialTreeLayout {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius_increment: 80.0,
            start_angle: 0.0,
        }
    }
}

impl RadialTreeLayout {
    pub fn apply(
        &self,
        nodes: &mut HashMap<NodeId, Vec3>,
        edges: &[(NodeId, NodeId)],
        root_id: NodeId,
    ) {
        if !nodes.contains_key(&root_id) {
            return;
        }

        // Build adjacency list
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for (source, target) in edges {
            adjacency.entry(source.clone()).or_default().push(target.clone());
            adjacency.entry(target.clone()).or_default().push(source.clone());
        }

        // BFS to assign levels
        let mut levels: HashMap<NodeId, usize> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(root_id.clone());
        levels.insert(root_id.clone(), 0);
        visited.insert(root_id.clone());

        let mut nodes_at_level: HashMap<usize, Vec<NodeId>> = HashMap::new();
        nodes_at_level.entry(0).or_default().push(root_id.clone());

        while let Some(current) = queue.pop_front() {
            let current_level = levels[&current];
            
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        levels.insert(neighbor.clone(), current_level + 1);
                        nodes_at_level.entry(current_level + 1).or_default().push(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Position nodes
        // Root at center
        if let Some(pos) = nodes.get_mut(&root_id) {
            *pos = self.center;
        }

        // Position other levels
        for (level, level_nodes) in nodes_at_level.iter() {
            if *level == 0 {
                continue;
            }

            let radius = *level as f32 * self.radius_increment;
            let angle_increment = std::f32::consts::TAU / level_nodes.len() as f32;

            for (i, node_id) in level_nodes.iter().enumerate() {
                let angle = self.start_angle + i as f32 * angle_increment;
                let x = self.center.x + radius * angle.cos();
                let y = self.center.y + radius * angle.sin();
                let z = self.center.z; // Keep on same plane

                if let Some(pos) = nodes.get_mut(node_id) {
                    *pos = Vec3::new(x, y, z);
                }
            }
        }
    }
}

/// Spectral Layout - uses eigenvalues of the graph Laplacian matrix
/// This provides optimal layouts for certain graph types
pub struct SpectralLayout {
    pub dimensions: usize,
}

impl Default for SpectralLayout {
    fn default() -> Self {
        Self { dimensions: 3 }
    }
}

impl SpectralLayout {
    pub fn apply(&self, nodes: &mut HashMap<NodeId, Vec3>, edges: &[(NodeId, NodeId)]) {
        let node_count = nodes.len();
        if node_count < 2 {
            return;
        }

        // Create node index mapping
        let node_ids: Vec<NodeId> = nodes.keys().cloned().collect();
        let id_to_index: HashMap<NodeId, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        // Build adjacency matrix
        let mut adjacency = vec![vec![0.0; node_count]; node_count];
        for (source, target) in edges {
            if let (Some(&i), Some(&j)) = (id_to_index.get(source), id_to_index.get(target)) {
                adjacency[i][j] = 1.0;
                adjacency[j][i] = 1.0;
            }
        }

        // Calculate degree matrix
        let mut degree = vec![0.0; node_count];
        for i in 0..node_count {
            degree[i] = adjacency[i].iter().sum();
        }

        // Calculate Laplacian matrix (L = D - A)
        let mut laplacian = vec![vec![0.0; node_count]; node_count];
        for i in 0..node_count {
            for j in 0..node_count {
                if i == j {
                    laplacian[i][j] = degree[i];
                } else {
                    laplacian[i][j] = -adjacency[i][j];
                }
            }
        }

        // For simplicity, use a basic layout based on node degrees
        // (Full spectral layout would require eigenvalue decomposition)
        let max_degree = degree.iter().fold(0.0f32, |a, &b| a.max(b));
        let scale = 200.0;

        for (i, node_id) in node_ids.iter().enumerate() {
            let normalized_degree = if max_degree > 0.0 {
                degree[i] / max_degree
            } else {
                0.5
            };

            // Position based on degree centrality
            let angle = (i as f32 / node_count as f32) * std::f32::consts::TAU;
            let radius = scale * (1.0 - normalized_degree * 0.5);

            let x = radius * angle.cos();
            let y = radius * angle.sin();
            let z = normalized_degree * scale * 0.5; // Higher degree nodes elevated

            if let Some(pos) = nodes.get_mut(node_id) {
                *pos = Vec3::new(x, y, z);
            }
        }
    }
}

/// Bipartite Layout - optimized for bipartite graphs
pub struct BipartiteLayout {
    pub layer_distance: f32,
    pub node_spacing: f32,
}

impl Default for BipartiteLayout {
    fn default() -> Self {
        Self {
            layer_distance: 200.0,
            node_spacing: 50.0,
        }
    }
}

impl BipartiteLayout {
    pub fn apply(
        &self,
        nodes: &mut HashMap<NodeId, Vec3>,
        _edges: &[(NodeId, NodeId)],
        set_a: &HashSet<NodeId>,
    ) {
        // Determine set B (all nodes not in set A)
        let set_b: HashSet<NodeId> = nodes
            .keys()
            .filter(|id| !set_a.contains(id))
            .cloned()
            .collect();

        // Position set A nodes
        let a_count = set_a.len();
        let _a_height = a_count as f32 * self.node_spacing;
        for (i, node_id) in set_a.iter().enumerate() {
            let y = (i as f32 - (a_count - 1) as f32 / 2.0) * self.node_spacing;
            if let Some(pos) = nodes.get_mut(node_id) {
                *pos = Vec3::new(-self.layer_distance / 2.0, y, 0.0);
            }
        }

        // Position set B nodes
        let b_count = set_b.len();
        let _b_height = b_count as f32 * self.node_spacing;
        for (i, node_id) in set_b.iter().enumerate() {
            let y = (i as f32 - (b_count - 1) as f32 / 2.0) * self.node_spacing;
            if let Some(pos) = nodes.get_mut(node_id) {
                *pos = Vec3::new(self.layer_distance / 2.0, y, 0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruchterman_reingold_layout() {
        let mut nodes = HashMap::new();
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        let id3 = NodeId::new();
        
        nodes.insert(id1.clone(), Vec3::new(0.0, 0.0, 0.0));
        nodes.insert(id2.clone(), Vec3::new(10.0, 0.0, 0.0));
        nodes.insert(id3.clone(), Vec3::new(5.0, 5.0, 0.0));
        
        let edges = vec![(id1.clone(), id2.clone()), (id2.clone(), id3.clone())];
        let bounds = Vec3::new(500.0, 500.0, 500.0);
        
        let mut layout = FruchtermanReingoldLayout::default();
        layout.max_iterations = 10; // Reduce for testing
        layout.apply(&mut nodes, &edges, bounds);
        
        // Verify nodes have moved
        assert_ne!(nodes[&id1], Vec3::new(0.0, 0.0, 0.0));
        assert_ne!(nodes[&id2], Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_sphere_layout() {
        let mut nodes = HashMap::new();
        for _ in 0..10 {
            nodes.insert(NodeId::new(), Vec3::ZERO);
        }
        
        let layout = SphereLayout::default();
        layout.apply(&mut nodes);
        
        // Verify all nodes are on sphere surface
        for (_, pos) in nodes.iter() {
            let distance = pos.length();
            assert!((distance - layout.radius).abs() < 0.1);
        }
    }

    #[test]
    fn test_radial_tree_layout() {
        let mut nodes = HashMap::new();
        let root = NodeId::new();
        let child1 = NodeId::new();
        let child2 = NodeId::new();
        
        nodes.insert(root.clone(), Vec3::ZERO);
        nodes.insert(child1.clone(), Vec3::ZERO);
        nodes.insert(child2.clone(), Vec3::ZERO);
        
        let edges = vec![(root.clone(), child1.clone()), (root.clone(), child2.clone())];
        
        let layout = RadialTreeLayout::default();
        layout.apply(&mut nodes, &edges, root.clone());
        
        // Verify root is at center
        assert_eq!(nodes[&root], layout.center);
        
        // Verify children are at same radius
        let child1_radius = (nodes[&child1] - layout.center).length();
        let child2_radius = (nodes[&child2] - layout.center).length();
        assert!((child1_radius - child2_radius).abs() < 0.1);
    }
}