//! Level of Detail (LOD) system for efficient rendering of distant nodes
//!
//! Renders simplified representations of nodes based on distance from camera,
//! reducing GPU load while maintaining visual quality.

use bevy_ecs::prelude::*;
use crate::components::{NodeEntity, visual::Position3D};

/// LOD levels for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum LodLevel {
    /// Full detail - close to camera
    High = 0,
    /// Medium detail - moderate distance
    Medium = 1,
    /// Low detail - far from camera
    Low = 2,
    /// Very low detail - very far
    Minimal = 3,
    /// Not rendered - beyond max distance
    Culled = 4,
}

impl LodLevel {
    /// Get the complexity factor for this LOD level (0.0 to 1.0)
    pub fn complexity_factor(&self) -> f32 {
        match self {
            LodLevel::High => 1.0,
            LodLevel::Medium => 0.5,
            LodLevel::Low => 0.25,
            LodLevel::Minimal => 0.1,
            LodLevel::Culled => 0.0,
        }
    }

    /// Get the vertex count multiplier for mesh simplification
    pub fn vertex_multiplier(&self) -> f32 {
        match self {
            LodLevel::High => 1.0,
            LodLevel::Medium => 0.3,
            LodLevel::Low => 0.1,
            LodLevel::Minimal => 0.05,
            LodLevel::Culled => 0.0,
        }
    }

    /// Whether edges should be rendered at this LOD level
    pub fn render_edges(&self) -> bool {
        matches!(self, LodLevel::High | LodLevel::Medium)
    }

    /// Whether labels should be rendered at this LOD level
    pub fn render_labels(&self) -> bool {
        matches!(self, LodLevel::High)
    }
}

/// Configuration for LOD system
#[derive(Resource, Debug, Clone)]
pub struct LodConfig {
    /// Camera position for distance calculations
    pub camera_position: [f32; 3],
    /// Distance thresholds for each LOD level
    pub distances: [f32; 4],
    /// Whether to use squared distances for performance
    pub use_squared_distances: bool,
    /// Hysteresis factor to prevent LOD flickering
    pub hysteresis: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            camera_position: [0.0, 0.0, 0.0],
            distances: [100.0, 500.0, 1000.0, 2000.0],
            use_squared_distances: true,
            hysteresis: 1.1,
        }
    }
}

/// System to update LOD levels based on distance from camera
pub fn update_lod_system(
    config: Res<LodConfig>,
    mut nodes: Query<(&Position3D, &mut LodLevel), With<NodeEntity>>,
) {
    let cam_pos = config.camera_position;
    let distances_squared = if config.use_squared_distances {
        [
            config.distances[0] * config.distances[0],
            config.distances[1] * config.distances[1],
            config.distances[2] * config.distances[2],
            config.distances[3] * config.distances[3],
        ]
    } else {
        config.distances
    };

    for (position, mut lod) in nodes.iter_mut() {
        // Calculate distance from camera
        let dx = position.x as f32 - cam_pos[0];
        let dy = position.y as f32 - cam_pos[1];
        let dz = position.z as f32 - cam_pos[2];
        
        let dist_squared = dx * dx + dy * dy + dz * dz;
        let distance = if config.use_squared_distances {
            dist_squared
        } else {
            dist_squared.sqrt()
        };

        // Determine new LOD level
        let new_lod = if distance < distances_squared[0] {
            LodLevel::High
        } else if distance < distances_squared[1] {
            LodLevel::Medium
        } else if distance < distances_squared[2] {
            LodLevel::Low
        } else if distance < distances_squared[3] {
            LodLevel::Minimal
        } else {
            LodLevel::Culled
        };

        // Apply hysteresis to prevent flickering
        if should_change_lod(&lod, new_lod, distance, &distances_squared, config.hysteresis) {
            *lod = new_lod;
        }
    }
}

/// Check if LOD should change, considering hysteresis
fn should_change_lod(
    current: &LodLevel,
    new: LodLevel,
    distance: f32,
    thresholds: &[f32; 4],
    hysteresis: f32,
) -> bool {
    if current == &new {
        return false;
    }

    let current_idx = *current as usize;
    let new_idx = new as usize;

    // Moving to lower detail (further away)
    if new_idx > current_idx {
        return true;
    }

    // Moving to higher detail (closer) - apply hysteresis
    if new_idx < current_idx && current_idx > 0 {
        let threshold = thresholds[current_idx - 1] / hysteresis;
        return distance < threshold;
    }

    true
}

/// Stats for LOD system
#[derive(Resource, Default, Debug)]
pub struct LodStats {
    pub high_detail_count: usize,
    pub medium_detail_count: usize,
    pub low_detail_count: usize,
    pub minimal_detail_count: usize,
    pub culled_count: usize,
    pub total_count: usize,
}

/// System to update LOD statistics
pub fn update_lod_stats(
    nodes: Query<&LodLevel, With<NodeEntity>>,
    mut stats: ResMut<LodStats>,
) {
    stats.high_detail_count = 0;
    stats.medium_detail_count = 0;
    stats.low_detail_count = 0;
    stats.minimal_detail_count = 0;
    stats.culled_count = 0;
    stats.total_count = 0;

    for lod in nodes.iter() {
        stats.total_count += 1;
        match lod {
            LodLevel::High => stats.high_detail_count += 1,
            LodLevel::Medium => stats.medium_detail_count += 1,
            LodLevel::Low => stats.low_detail_count += 1,
            LodLevel::Minimal => stats.minimal_detail_count += 1,
            LodLevel::Culled => stats.culled_count += 1,
        }
    }
}

/// Bundle for LOD components
#[derive(Bundle)]
pub struct LodBundle {
    pub lod_level: LodLevel,
}

impl Default for LodBundle {
    fn default() -> Self {
        Self {
            lod_level: LodLevel::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_levels() {
        assert_eq!(LodLevel::High.complexity_factor(), 1.0);
        assert_eq!(LodLevel::Medium.complexity_factor(), 0.5);
        assert_eq!(LodLevel::Low.complexity_factor(), 0.25);
        assert_eq!(LodLevel::Minimal.complexity_factor(), 0.1);
        assert_eq!(LodLevel::Culled.complexity_factor(), 0.0);

        assert!(LodLevel::High.render_edges());
        assert!(LodLevel::Medium.render_edges());
        assert!(!LodLevel::Low.render_edges());

        assert!(LodLevel::High.render_labels());
        assert!(!LodLevel::Medium.render_labels());
    }

    #[test]
    fn test_hysteresis() {
        let thresholds = [100.0, 500.0, 1000.0, 2000.0];
        let hysteresis = 1.2;

        // Should change when moving to lower detail
        assert!(should_change_lod(
            &LodLevel::High,
            LodLevel::Medium,
            150.0,
            &thresholds,
            hysteresis
        ));

        // Should not change back immediately due to hysteresis
        assert!(!should_change_lod(
            &LodLevel::Medium,
            LodLevel::High,
            90.0,
            &thresholds,
            hysteresis
        ));

        // Should change when well below threshold
        assert!(should_change_lod(
            &LodLevel::Medium,
            LodLevel::High,
            80.0,
            &thresholds,
            hysteresis
        ));
    }
}