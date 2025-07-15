//! Batched rendering system for efficient GPU utilization
//!
//! Groups similar nodes and edges for instanced rendering,
//! dramatically reducing draw calls for large graphs.

use bevy_ecs::prelude::*;
use bevy_app::Plugin;
use std::collections::HashMap;
use crate::components::{NodeEntity, EdgeEntity, NodeType};
use crate::performance::level_of_detail::LodLevel;

/// Batch key for grouping similar render items
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BatchKey {
    /// Type of element (node type or edge)
    pub element_type: ElementType,
    /// LOD level
    pub lod_level: LodLevel,
    /// Material/color key
    pub material_key: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementType {
    Node(NodeType),
    Edge,
}

/// Instance data for a single rendered element
#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    /// World transform matrix (4x4)
    pub transform: [[f32; 4]; 4],
    /// Color (RGBA)
    pub color: [f32; 4],
    /// Custom data (e.g., selection state, hover)
    pub custom: [f32; 4],
}

/// A batch of instances to render together
#[derive(Debug, Default)]
pub struct RenderBatch {
    /// The key identifying this batch
    pub key: Option<BatchKey>,
    /// Instance data for all elements in this batch
    pub instances: Vec<InstanceData>,
    /// Entity IDs for picking/selection
    pub entities: Vec<Entity>,
}

impl RenderBatch {
    /// Create a new empty batch
    pub fn new(key: BatchKey) -> Self {
        Self {
            key: Some(key),
            instances: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Add an instance to the batch
    pub fn add_instance(&mut self, entity: Entity, instance: InstanceData) {
        self.instances.push(instance);
        self.entities.push(entity);
    }

    /// Get the number of instances in this batch
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

/// Resource containing all render batches
#[derive(Resource, Default)]
pub struct RenderBatches {
    /// All batches organized by key
    pub batches: HashMap<BatchKey, RenderBatch>,
    /// Statistics
    pub stats: BatchingStats,
}

#[derive(Debug, Default)]
pub struct BatchingStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub batch_count: usize,
    pub largest_batch: usize,
    pub draw_calls_saved: usize,
}

impl RenderBatches {
    /// Clear all batches
    pub fn clear(&mut self) {
        self.batches.clear();
        self.stats = BatchingStats::default();
    }

    /// Add an instance to the appropriate batch
    pub fn add_instance(&mut self, key: BatchKey, entity: Entity, instance: InstanceData) {
        let batch = self.batches.entry(key.clone()).or_insert_with(|| RenderBatch::new(key));
        batch.add_instance(entity, instance);
    }

    /// Update statistics
    pub fn update_stats(&mut self) {
        self.stats.batch_count = self.batches.len();
        self.stats.total_nodes = 0;
        self.stats.total_edges = 0;
        self.stats.largest_batch = 0;

        for (key, batch) in &self.batches {
            let count = batch.len();
            self.stats.largest_batch = self.stats.largest_batch.max(count);
            
            match &key.element_type {
                ElementType::Node(_) => self.stats.total_nodes += count,
                ElementType::Edge => self.stats.total_edges += count,
            }
        }

        // Calculate draw calls saved (would be 1 per element without batching)
        let total_elements = self.stats.total_nodes + self.stats.total_edges;
        self.stats.draw_calls_saved = total_elements.saturating_sub(self.stats.batch_count);
    }
}

/// System to batch nodes for rendering
pub fn batch_nodes_system(
    mut batches: ResMut<RenderBatches>,
    nodes: Query<(
        Entity,
        &crate::components::visual::Position3D,
        &NodeType,
        &LodLevel,
        Option<&crate::components::visual::Color>,
    ), With<NodeEntity>>,
) {
    batches.clear();

    for (entity, position, node_type, lod_level, color) in nodes.iter() {
        // Skip culled nodes
        if *lod_level == LodLevel::Culled {
            continue;
        }

        // Create batch key
        let material_key = color
            .map(|c| ((c.r * 255.0) as u32) << 24 | ((c.g * 255.0) as u32) << 16 | ((c.b * 255.0) as u32) << 8 | ((c.a * 255.0) as u32))
            .unwrap_or(0xFFFFFFFF);

        let key = BatchKey {
            element_type: ElementType::Node(node_type.clone()),
            lod_level: *lod_level,
            material_key,
        };

        // Create instance data
        let transform = create_transform_matrix(
            position.x as f32,
            position.y as f32,
            position.z as f32,
            1.0, // scale
        );

        let color_array = color
            .map(|c| [c.r, c.g, c.b, c.a])
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);

        let instance = InstanceData {
            transform,
            color: color_array,
            custom: [0.0; 4], // Could store selection state, etc.
        };

        batches.add_instance(key, entity, instance);
    }

    batches.update_stats();
}

/// System to batch edges for rendering
pub fn batch_edges_system(
    mut batches: ResMut<RenderBatches>,
    edges: Query<(
        Entity,
        &EdgeEntity,
        Option<&crate::components::visual::Color>,
    )>,
    nodes: Query<&crate::components::visual::Position3D>,
) {
    for (entity, _edge, color) in edges.iter() {
        // Get positions of connected nodes
        let source_pos = nodes.get(Entity::from_raw(0)).ok(); // Would need proper entity lookup
        let target_pos = nodes.get(Entity::from_raw(0)).ok();

        if source_pos.is_none() || target_pos.is_none() {
            continue;
        }

        let material_key = color
            .map(|c| ((c.r * 255.0) as u32) << 24 | ((c.g * 255.0) as u32) << 16 | ((c.b * 255.0) as u32) << 8 | ((c.a * 255.0) as u32))
            .unwrap_or(0x808080FF);

        let key = BatchKey {
            element_type: ElementType::Edge,
            lod_level: LodLevel::High, // Edges only rendered at high LOD
            material_key,
        };

        // For edges, transform would encode start/end positions
        let transform = [[0.0; 4]; 4]; // Simplified

        let color_array = color
            .map(|c| [c.r, c.g, c.b, c.a])
            .unwrap_or([0.5, 0.5, 0.5, 1.0]);

        let instance = InstanceData {
            transform,
            color: color_array,
            custom: [0.0; 4],
        };

        batches.add_instance(key, entity, instance);
    }
}

/// Create a 4x4 transform matrix
fn create_transform_matrix(x: f32, y: f32, z: f32, scale: f32) -> [[f32; 4]; 4] {
    [
        [scale, 0.0, 0.0, 0.0],
        [0.0, scale, 0.0, 0.0],
        [0.0, 0.0, scale, 0.0],
        [x, y, z, 1.0],
    ]
}

/// Plugin for batched rendering
pub struct BatchedRenderingPlugin;

impl Plugin for BatchedRenderingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<RenderBatches>()
            .add_systems(
                bevy_app::Update,
                (batch_nodes_system, batch_edges_system).chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_batch() {
        let mut batch = RenderBatch::new(BatchKey {
            element_type: ElementType::Node(NodeType::default()),
            lod_level: LodLevel::High,
            material_key: 0xFFFFFFFF,
        });

        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);

        let instance = InstanceData {
            transform: [[1.0; 4]; 4],
            color: [1.0, 0.0, 0.0, 1.0],
            custom: [0.0; 4],
        };

        batch.add_instance(Entity::from_raw(1), instance);
        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_render_batches() {
        let mut batches = RenderBatches::default();

        let key1 = BatchKey {
            element_type: ElementType::Node(NodeType::default()),
            lod_level: LodLevel::High,
            material_key: 0xFF0000FF,
        };

        let key2 = BatchKey {
            element_type: ElementType::Edge,
            lod_level: LodLevel::High,
            material_key: 0x00FF00FF,
        };

        let instance = InstanceData {
            transform: [[1.0; 4]; 4],
            color: [1.0, 0.0, 0.0, 1.0],
            custom: [0.0; 4],
        };

        batches.add_instance(key1, Entity::from_raw(1), instance);
        batches.add_instance(key2, Entity::from_raw(2), instance);

        batches.update_stats();

        assert_eq!(batches.stats.batch_count, 2);
        assert_eq!(batches.stats.total_nodes, 1);
        assert_eq!(batches.stats.total_edges, 1);
    }
}