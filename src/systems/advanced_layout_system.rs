//! Advanced layout system integration for Bevy
//!
//! This module integrates the advanced layout algorithms with the Bevy ECS system

use bevy_ecs::prelude::*;
use bevy_app::Plugin;
use std::collections::{HashMap, HashSet};
use crate::NodeId;
use crate::components::{NodeEntity, EdgeEntity};
use crate::layout::{
    FruchtermanReingoldLayout, SphereLayout, RadialTreeLayout,
    SpectralLayout, BipartiteLayout, advanced_layouts::Vec3
};

/// Advanced layout types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdvancedLayoutType {
    FruchtermanReingold,
    Sphere,
    RadialTree,
    Spectral,
    Bipartite,
}

/// Advanced layout configuration
#[derive(Component, Debug, Clone)]
pub struct AdvancedLayoutConfig {
    pub layout_type: AdvancedLayoutType,
    pub bounds: Vec3,
    /// For radial tree layout
    pub root_node: Option<NodeId>,
    /// For bipartite layout
    pub bipartite_set_a: HashSet<NodeId>,
}

impl Default for AdvancedLayoutConfig {
    fn default() -> Self {
        Self {
            layout_type: AdvancedLayoutType::FruchtermanReingold,
            bounds: Vec3::new(1000.0, 1000.0, 1000.0),
            root_node: None,
            bipartite_set_a: HashSet::new(),
        }
    }
}

/// Event to trigger advanced layout
#[derive(Event)]
pub struct ApplyAdvancedLayout {
    pub layout_type: AdvancedLayoutType,
    pub config: AdvancedLayoutConfig,
}

/// System to apply advanced layouts
pub fn apply_advanced_layout_system(
    mut events: EventReader<ApplyAdvancedLayout>,
    mut node_query: Query<(&NodeEntity, &mut crate::components::visual::Position3D)>,
    edge_query: Query<&EdgeEntity>,
) {
    for event in events.read() {
        // Collect node positions
        let mut node_positions: HashMap<NodeId, Vec3> = HashMap::new();
        for (node, position) in node_query.iter() {
            node_positions.insert(node.node_id.clone(), Vec3::new(position.x as f32, position.y as f32, position.z as f32));
        }

        // Collect edges
        let edges: Vec<(NodeId, NodeId)> = edge_query
            .iter()
            .map(|edge| (edge.source.clone(), edge.target.clone()))
            .collect();

        // Apply the appropriate layout
        match event.layout_type {
            AdvancedLayoutType::FruchtermanReingold => {
                let mut layout = FruchtermanReingoldLayout::default();
                layout.apply(&mut node_positions, &edges, event.config.bounds);
            }
            AdvancedLayoutType::Sphere => {
                let layout = SphereLayout::default();
                layout.apply(&mut node_positions);
            }
            AdvancedLayoutType::RadialTree => {
                if let Some(root) = event.config.root_node {
                    let layout = RadialTreeLayout::default();
                    layout.apply(&mut node_positions, &edges, root);
                }
            }
            AdvancedLayoutType::Spectral => {
                let layout = SpectralLayout::default();
                layout.apply(&mut node_positions, &edges);
            }
            AdvancedLayoutType::Bipartite => {
                let layout = BipartiteLayout::default();
                layout.apply(&mut node_positions, &edges, &event.config.bipartite_set_a);
            }
        }

        // Apply new positions to transforms
        for (node, mut position) in node_query.iter_mut() {
            if let Some(&new_pos) = node_positions.get(&node.node_id) {
                position.x = new_pos.x as f64;
                position.y = new_pos.y as f64;
                position.z = new_pos.z as f64;
            }
        }
    }
}

/// Layout quality metrics
#[derive(Debug, Clone, Resource)]
pub struct LayoutQualityMetrics {
    pub edge_length_variance: f32,
    pub node_overlap_count: usize,
    pub edge_crossing_count: usize,
    pub aspect_ratio: f32,
    pub node_distribution_score: f32,
}

/// System to calculate layout quality metrics
pub fn calculate_layout_quality_system(
    node_query: Query<(&NodeEntity, &crate::components::visual::Position3D)>,
    edge_query: Query<&EdgeEntity>,
    mut metrics: ResMut<LayoutQualityMetrics>,
) {
    let nodes: Vec<(NodeId, Vec3)> = node_query
        .iter()
        .map(|(node, position)| (node.node_id.clone(), Vec3::new(position.x as f32, position.y as f32, position.z as f32)))
        .collect();

    let node_map: HashMap<NodeId, Vec3> = nodes.iter().cloned().collect();

    // Calculate edge length variance
    let edge_lengths: Vec<f32> = edge_query
        .iter()
        .filter_map(|edge| {
            let source_pos = node_map.get(&edge.source)?;
            let target_pos = node_map.get(&edge.target)?;
            Some((*source_pos - *target_pos).length())
        })
        .collect();

    if !edge_lengths.is_empty() {
        let mean_length = edge_lengths.iter().sum::<f32>() / edge_lengths.len() as f32;
        let variance = edge_lengths
            .iter()
            .map(|&len| (len - mean_length).powi(2))
            .sum::<f32>() / edge_lengths.len() as f32;
        metrics.edge_length_variance = variance.sqrt();
    }

    // Count node overlaps (simplified - checks if nodes are too close)
    let overlap_threshold = 30.0;
    let mut overlap_count = 0;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let distance = (nodes[i].1 - nodes[j].1).length();
            if distance < overlap_threshold {
                overlap_count += 1;
            }
        }
    }
    metrics.node_overlap_count = overlap_count;

    // Calculate bounding box and aspect ratio
    if !nodes.is_empty() {
        let min_x = nodes.iter().map(|(_, pos)| pos.x).fold(f32::INFINITY, f32::min);
        let max_x = nodes.iter().map(|(_, pos)| pos.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = nodes.iter().map(|(_, pos)| pos.y).fold(f32::INFINITY, f32::min);
        let max_y = nodes.iter().map(|(_, pos)| pos.y).fold(f32::NEG_INFINITY, f32::max);

        let width = max_x - min_x;
        let height = max_y - min_y;
        metrics.aspect_ratio = if height > 0.0 { width / height } else { 1.0 };
    }

    // Calculate node distribution score (0-1, higher is better)
    // Uses coefficient of variation of nearest neighbor distances
    if nodes.len() > 1 {
        let mut nearest_distances = Vec::new();
        for i in 0..nodes.len() {
            let mut min_distance = f32::INFINITY;
            for j in 0..nodes.len() {
                if i != j {
                    let distance = (nodes[i].1 - nodes[j].1).length();
                    min_distance = min_distance.min(distance);
                }
            }
            nearest_distances.push(min_distance);
        }

        let mean_distance = nearest_distances.iter().sum::<f32>() / nearest_distances.len() as f32;
        let variance = nearest_distances
            .iter()
            .map(|&d| (d - mean_distance).powi(2))
            .sum::<f32>() / nearest_distances.len() as f32;
        let cv = variance.sqrt() / mean_distance;
        metrics.node_distribution_score = 1.0 / (1.0 + cv);
    }
}

/// Resource wrapper for layout quality metrics
#[derive(Resource, Debug, Clone)]
pub struct LayoutQualityMetricsResource(pub LayoutQualityMetrics);

/// Plugin for advanced layout systems
pub struct AdvancedLayoutPlugin;

impl Plugin for AdvancedLayoutPlugin {
    fn build(&self, _app: &mut bevy_app::App) {
        // Note: This plugin needs to be registered in a Bevy app context
        // The actual implementation would be in cim-domain-bevy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_layout_config() {
        let config = AdvancedLayoutConfig::default();
        assert_eq!(config.layout_type, AdvancedLayoutType::FruchtermanReingold);
        assert_eq!(config.bounds, Vec3::new(1000.0, 1000.0, 1000.0));
    }

    #[test]
    fn test_layout_quality_metrics() {
        let metrics = LayoutQualityMetrics {
            edge_length_variance: 10.0,
            node_overlap_count: 2,
            edge_crossing_count: 5,
            aspect_ratio: 1.5,
            node_distribution_score: 0.8,
        };
        
        assert_eq!(metrics.node_overlap_count, 2);
        assert_eq!(metrics.node_distribution_score, 0.8);
    }
}