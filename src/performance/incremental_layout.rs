//! Incremental layout updates for dynamic graphs
//!
//! Instead of recalculating the entire layout when changes occur,
//! this system only updates the affected regions of the graph.

use std::collections::{HashMap, HashSet, VecDeque};
use bevy_ecs::prelude::*;
use crate::{NodeId, EdgeId};
use crate::components::NodeEntity;
use crate::layout::advanced_layouts::Vec3;

/// Tracks changes to the graph structure
#[derive(Resource, Default)]
pub struct GraphChangeTracker {
    /// Nodes that were added since last layout
    pub added_nodes: HashSet<NodeId>,
    /// Nodes that were removed
    pub removed_nodes: HashSet<NodeId>,
    /// Nodes that moved significantly
    pub moved_nodes: HashSet<NodeId>,
    /// Edges that were added
    pub added_edges: HashSet<EdgeId>,
    /// Edges that were removed
    pub removed_edges: HashSet<EdgeId>,
    /// Nodes affected by changes (need layout update)
    pub affected_nodes: HashSet<NodeId>,
    /// Whether a full relayout is needed
    pub needs_full_relayout: bool,
}

/// Resource to track Entity to NodeId mappings
#[derive(Resource, Default)]
pub struct EntityNodeMapping {
    pub entity_to_node: HashMap<Entity, NodeId>,
    pub node_to_entity: HashMap<NodeId, Entity>,
}

impl GraphChangeTracker {
    /// Clear all tracked changes
    pub fn clear(&mut self) {
        self.added_nodes.clear();
        self.removed_nodes.clear();
        self.moved_nodes.clear();
        self.added_edges.clear();
        self.removed_edges.clear();
        self.affected_nodes.clear();
        self.needs_full_relayout = false;
    }

    /// Check if any changes occurred
    pub fn has_changes(&self) -> bool {
        !self.added_nodes.is_empty()
            || !self.removed_nodes.is_empty()
            || !self.moved_nodes.is_empty()
            || !self.added_edges.is_empty()
            || !self.removed_edges.is_empty()
    }

    /// Get the total number of affected nodes
    pub fn affected_count(&self) -> usize {
        self.affected_nodes.len()
    }

    /// Determine if changes warrant a full relayout
    pub fn should_full_relayout(&self, total_nodes: usize) -> bool {
        if self.needs_full_relayout {
            return true;
        }

        // If more than 30% of nodes are affected, do full relayout
        let affected_ratio = self.affected_nodes.len() as f32 / total_nodes.max(1) as f32;
        affected_ratio > 0.3
    }
}

/// Configuration for incremental layout
#[derive(Resource)]
pub struct IncrementalLayoutConfig {
    /// Maximum distance to propagate changes
    pub propagation_distance: usize,
    /// Movement threshold to trigger update
    pub movement_threshold: f32,
    /// Whether to use adaptive time stepping
    pub adaptive_timestep: bool,
    /// Base time step for layout iterations
    pub base_timestep: f32,
    /// Maximum iterations per frame
    pub max_iterations_per_frame: usize,
}

impl Default for IncrementalLayoutConfig {
    fn default() -> Self {
        Self {
            propagation_distance: 3,
            movement_threshold: 10.0,
            adaptive_timestep: true,
            base_timestep: 0.1,
            max_iterations_per_frame: 50,
        }
    }
}

/// Cached layout state for incremental updates
#[derive(Resource, Default)]
pub struct LayoutCache {
    /// Node positions from last frame
    pub previous_positions: HashMap<NodeId, Vec3>,
    /// Node velocities for momentum
    pub velocities: HashMap<NodeId, Vec3>,
    /// Adjacency list for quick neighbor lookup
    pub adjacency: HashMap<NodeId, Vec<NodeId>>,
    /// Node degrees (number of connections)
    pub degrees: HashMap<NodeId, usize>,
    /// Pinned nodes that shouldn't move
    pub pinned_nodes: HashSet<NodeId>,
}

impl LayoutCache {
    /// Update adjacency list from edges
    pub fn update_adjacency(&mut self, edges: &[(NodeId, NodeId)]) {
        self.adjacency.clear();
        self.degrees.clear();

        for (source, target) in edges {
            self.adjacency.entry(source.clone()).or_default().push(target.clone());
            self.adjacency.entry(target.clone()).or_default().push(source.clone());
            
            *self.degrees.entry(source.clone()).or_insert(0) += 1;
            *self.degrees.entry(target.clone()).or_insert(0) += 1;
        }
    }

    /// Pin a node to prevent movement
    pub fn pin_node(&mut self, node_id: NodeId) {
        self.pinned_nodes.insert(node_id);
        self.velocities.remove(&node_id);
    }

    /// Unpin a node to allow movement
    pub fn unpin_node(&mut self, node_id: &NodeId) {
        self.pinned_nodes.remove(node_id);
    }

    /// Check if a node is pinned
    pub fn is_pinned(&self, node_id: &NodeId) -> bool {
        self.pinned_nodes.contains(node_id)
    }
}

/// Incremental force-directed layout
pub struct IncrementalForceLayout {
    /// Current positions
    positions: HashMap<NodeId, Vec3>,
    /// Configuration
    config: IncrementalLayoutConfig,
    /// Layout cache
    cache: LayoutCache,
    /// Nodes to update this iteration
    update_queue: VecDeque<NodeId>,
}

impl IncrementalForceLayout {
    /// Create a new incremental layout
    pub fn new(
        positions: HashMap<NodeId, Vec3>,
        config: IncrementalLayoutConfig,
        cache: LayoutCache,
    ) -> Self {
        Self {
            positions,
            config,
            cache,
            update_queue: VecDeque::new(),
        }
    }

    /// Apply incremental layout update
    pub fn update(
        &mut self,
        changes: &GraphChangeTracker,
        edges: &[(NodeId, NodeId)],
    ) -> HashMap<NodeId, Vec3> {
        // Update adjacency if edges changed
        if !changes.added_edges.is_empty() || !changes.removed_edges.is_empty() {
            self.cache.update_adjacency(edges);
        }

        // Initialize new nodes
        for node_id in &changes.added_nodes {
            if !self.positions.contains_key(node_id) {
                // Place new nodes near their neighbors
                let position = self.find_initial_position(node_id);
                self.positions.insert(node_id.clone(), position);
                self.cache.velocities.insert(node_id.clone(), Vec3::ZERO);
            }
        }

        // Remove deleted nodes
        for node_id in &changes.removed_nodes {
            self.positions.remove(node_id);
            self.cache.velocities.remove(node_id);
            self.cache.previous_positions.remove(node_id);
        }

        // Build update queue from affected nodes
        self.update_queue.clear();
        self.update_queue.extend(changes.affected_nodes.iter().cloned());

        // Propagate changes to neighbors
        let mut visited = HashSet::new();
        let mut current_distance = 0;

        while !self.update_queue.is_empty() && current_distance < self.config.propagation_distance {
            let level_size = self.update_queue.len();
            
            for _ in 0..level_size {
                if let Some(node_id) = self.update_queue.pop_front() {
                    if visited.insert(node_id.clone()) {
                        // Add neighbors to queue
                        if let Some(neighbors) = self.cache.adjacency.get(&node_id) {
                            for neighbor in neighbors {
                                if !visited.contains(neighbor) && !self.cache.is_pinned(neighbor) {
                                    self.update_queue.push_back(neighbor.clone());
                                }
                            }
                        }
                    }
                }
            }
            
            current_distance += 1;
        }

        // Apply forces only to nodes in visited set
        let mut iterations = 0;
        let max_iterations = self.config.max_iterations_per_frame;

        while iterations < max_iterations {
            let max_movement = self.apply_forces_incremental(&visited);
            
            // Adaptive timestep
            if self.config.adaptive_timestep && max_movement < self.config.movement_threshold {
                break;
            }
            
            iterations += 1;
        }

        // Update previous positions
        for (node_id, position) in &self.positions {
            self.cache.previous_positions.insert(node_id.clone(), *position);
        }

        self.positions.clone()
    }

    /// Find initial position for a new node
    fn find_initial_position(&self, node_id: &NodeId) -> Vec3 {
        // Place near neighbors if any
        if let Some(neighbors) = self.cache.adjacency.get(node_id) {
            if !neighbors.is_empty() {
                let mut center = Vec3::ZERO;
                let mut count = 0;

                for neighbor in neighbors {
                    if let Some(pos) = self.positions.get(neighbor) {
                        center = center + *pos;
                        count += 1;
                    }
                }

                if count > 0 {
                    center = center * (1.0 / count as f32);
                    // Add small random offset
                    center.x += (rand::random::<f32>() - 0.5) * 50.0;
                    center.y += (rand::random::<f32>() - 0.5) * 50.0;
                    center.z += (rand::random::<f32>() - 0.5) * 50.0;
                    return center;
                }
            }
        }

        // Random position if no neighbors
        Vec3::new(
            (rand::random::<f32>() - 0.5) * 1000.0,
            (rand::random::<f32>() - 0.5) * 1000.0,
            (rand::random::<f32>() - 0.5) * 1000.0,
        )
    }

    /// Apply forces to a subset of nodes
    fn apply_forces_incremental(&mut self, update_nodes: &HashSet<NodeId>) -> f32 {
        let k = 100.0; // Ideal spring length
        let k_squared = k * k;
        let timestep = self.config.base_timestep;
        let damping = 0.95;
        let mut max_movement: f32 = 0.0;

        // Calculate forces for each node in update set
        let mut forces: HashMap<NodeId, Vec3> = HashMap::new();

        for node_id in update_nodes {
            if self.cache.is_pinned(node_id) {
                continue;
            }

            if let Some(&pos1) = self.positions.get(node_id) {
                let mut force = Vec3::ZERO;

                // Repulsive forces from all nodes
                for (other_id, &pos2) in &self.positions {
                    if node_id != other_id {
                        let delta = pos1 - pos2;
                        let distance = delta.length().max(0.01);
                        let repulsion = k_squared / (distance * distance);
                        force = force + delta.normalize() * repulsion;
                    }
                }

                // Attractive forces from connected nodes
                if let Some(neighbors) = self.cache.adjacency.get(node_id) {
                    for neighbor in neighbors {
                        if let Some(&pos2) = self.positions.get(neighbor) {
                            let delta = pos2 - pos1;
                            let distance = delta.length().max(0.01);
                            let attraction = distance / k;
                            force = force + delta.normalize() * attraction;
                        }
                    }
                }

                forces.insert(node_id.clone(), force);
            }
        }

        // Apply forces and update velocities
        for (node_id, force) in forces {
            if let Some(velocity) = self.cache.velocities.get_mut(&node_id) {
                *velocity = (*velocity + force * timestep) * damping;
                
                if let Some(position) = self.positions.get_mut(&node_id) {
                    let movement = *velocity * timestep;
                    *position = *position + movement;
                    max_movement = max_movement.max(movement.length());
                }
            }
        }

        max_movement
    }
}

/// System to track graph changes
pub fn track_graph_changes_system(
    mut tracker: ResMut<GraphChangeTracker>,
    mut entity_mapping: ResMut<EntityNodeMapping>,
    added_nodes: Query<(Entity, &NodeEntity), Added<NodeEntity>>,
    mut removed_nodes: RemovedComponents<NodeEntity>,
    // Would need proper edge tracking
) {
    // Track added nodes and update entity mapping
    for (entity, node_entity) in added_nodes.iter() {
        let node_id = &node_entity.node_id;
        tracker.added_nodes.insert(node_id.clone());
        tracker.affected_nodes.insert(node_id.clone());
        
        // Update bidirectional mapping
        entity_mapping.entity_to_node.insert(entity, node_id.clone());
        entity_mapping.node_to_entity.insert(node_id.clone(), entity);
    }

    // Track removed nodes
    for entity in removed_nodes.read() {
        if let Some(node_id) = entity_mapping.entity_to_node.remove(&entity) {
            tracker.removed_nodes.insert(node_id.clone());
            tracker.affected_nodes.insert(node_id.clone());
            entity_mapping.node_to_entity.remove(&node_id);
        }
    }

    // TODO: Track edge changes and moved nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_tracker() {
        let mut tracker = GraphChangeTracker::default();
        
        assert!(!tracker.has_changes());
        
        let node_id = NodeId::new();
        tracker.added_nodes.insert(node_id.clone());
        tracker.affected_nodes.insert(node_id);
        
        assert!(tracker.has_changes());
        assert_eq!(tracker.affected_count(), 1);
        
        tracker.clear();
        assert!(!tracker.has_changes());
    }

    #[test]
    fn test_layout_cache() {
        let mut cache = LayoutCache::default();
        
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let edges = vec![(node1.clone(), node2.clone())];
        
        cache.update_adjacency(&edges);
        
        assert_eq!(cache.adjacency[&node1].len(), 1);
        assert_eq!(cache.adjacency[&node2].len(), 1);
        assert_eq!(cache.degrees[&node1], 1);
        assert_eq!(cache.degrees[&node2], 1);
        
        cache.pin_node(node1.clone());
        assert!(cache.is_pinned(&node1));
        assert!(!cache.is_pinned(&node2));
    }
}