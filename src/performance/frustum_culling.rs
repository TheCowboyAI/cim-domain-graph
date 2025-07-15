//! Frustum culling for efficient rendering of large graphs
//!
//! Only renders nodes that are within the camera's view frustum,
//! significantly reducing GPU load for large graphs.

use bevy_ecs::prelude::*;
use crate::components::{NodeEntity, visual::Position3D};

/// Represents a view frustum for culling
#[derive(Debug, Clone, Resource)]
pub struct ViewFrustum {
    /// Camera position
    pub position: [f32; 3],
    /// Camera forward direction
    pub forward: [f32; 3],
    /// Camera up direction
    pub up: [f32; 3],
    /// Field of view in radians
    pub fov: f32,
    /// Aspect ratio (width/height)
    pub aspect: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
    /// Cached frustum planes for efficient testing
    planes: [FrustumPlane; 6],
}

/// A plane in the view frustum
#[derive(Debug, Clone, Copy)]
struct FrustumPlane {
    normal: [f32; 3],
    distance: f32,
}

impl ViewFrustum {
    /// Create a new view frustum
    pub fn new(
        position: [f32; 3],
        forward: [f32; 3],
        up: [f32; 3],
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let mut frustum = Self {
            position,
            forward,
            up,
            fov,
            aspect,
            near,
            far,
            planes: [FrustumPlane { normal: [0.0, 0.0, 0.0], distance: 0.0 }; 6],
        };
        frustum.update_planes();
        frustum
    }

    /// Update the frustum planes after camera movement
    fn update_planes(&mut self) {
        // Calculate right vector
        let right = cross(self.forward, self.up);
        
        // Calculate half angles
        let half_v = (self.fov * 0.5).tan();
        let half_h = half_v * self.aspect;
        
        // Near and far plane centers
        let near_center = add(self.position, scale(self.forward, self.near));
        let far_center = add(self.position, scale(self.forward, self.far));
        
        // Near plane
        self.planes[0] = FrustumPlane {
            normal: self.forward,
            distance: -dot(self.forward, near_center),
        };
        
        // Far plane
        self.planes[1] = FrustumPlane {
            normal: scale(self.forward, -1.0),
            distance: -dot(scale(self.forward, -1.0), far_center),
        };
        
        // Calculate side planes
        let near_half_h = half_h * self.near;
        let near_half_v = half_v * self.near;
        
        // Right plane
        let right_normal = normalize(cross(
            sub(add(near_center, scale(right, near_half_h)), self.position),
            self.up
        ));
        self.planes[2] = FrustumPlane {
            normal: right_normal,
            distance: -dot(right_normal, self.position),
        };
        
        // Left plane
        let left_normal = normalize(cross(
            self.up,
            sub(sub(near_center, scale(right, near_half_h)), self.position)
        ));
        self.planes[3] = FrustumPlane {
            normal: left_normal,
            distance: -dot(left_normal, self.position),
        };
        
        // Top plane
        let top_normal = normalize(cross(
            right,
            sub(add(near_center, scale(self.up, near_half_v)), self.position)
        ));
        self.planes[4] = FrustumPlane {
            normal: top_normal,
            distance: -dot(top_normal, self.position),
        };
        
        // Bottom plane
        let bottom_normal = normalize(cross(
            sub(sub(near_center, scale(self.up, near_half_v)), self.position),
            right
        ));
        self.planes[5] = FrustumPlane {
            normal: bottom_normal,
            distance: -dot(bottom_normal, self.position),
        };
    }

    /// Test if a point is inside the frustum
    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        for plane in &self.planes {
            let distance = dot(plane.normal, point) + plane.distance;
            if distance < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is inside or intersects the frustum
    pub fn contains_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        for plane in &self.planes {
            let distance = dot(plane.normal, center) + plane.distance;
            if distance < -radius {
                return false;
            }
        }
        true
    }

    /// Test if a bounding box intersects the frustum
    pub fn intersects_aabb(&self, min: [f32; 3], max: [f32; 3]) -> bool {
        for plane in &self.planes {
            let mut p = [0.0; 3];
            
            // Find the positive vertex (furthest along the normal)
            for i in 0..3 {
                if plane.normal[i] >= 0.0 {
                    p[i] = max[i];
                } else {
                    p[i] = min[i];
                }
            }
            
            if dot(plane.normal, p) + plane.distance < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Component to mark entities as culled (not visible)
#[derive(Component, Default)]
pub struct Culled;

/// System to perform frustum culling on nodes
pub fn frustum_culling_system(
    frustum: Res<ViewFrustum>,
    mut commands: Commands,
    nodes: Query<(Entity, &Position3D), With<NodeEntity>>,
    culled: Query<Entity, With<Culled>>,
) {
    // Remove culled marker from previously culled entities
    for entity in culled.iter() {
        commands.entity(entity).remove::<Culled>();
    }
    
    // Check each node against the frustum
    for (entity, position) in nodes.iter() {
        let point = [position.x as f32, position.y as f32, position.z as f32];
        
        // Use sphere test with a reasonable radius for nodes
        const NODE_RADIUS: f32 = 10.0;
        if !frustum.contains_sphere(point, NODE_RADIUS) {
            commands.entity(entity).insert(Culled);
        }
    }
}

/// Stats tracking for frustum culling
#[derive(Resource, Default)]
pub struct FrustumCullingStats {
    pub total_nodes: usize,
    pub culled_nodes: usize,
    pub visible_nodes: usize,
}

/// System to update frustum culling statistics
pub fn update_frustum_stats(
    nodes: Query<&NodeEntity>,
    culled: Query<&Culled>,
    mut stats: ResMut<FrustumCullingStats>,
) {
    stats.total_nodes = nodes.iter().count();
    stats.culled_nodes = culled.iter().count();
    stats.visible_nodes = stats.total_nodes - stats.culled_nodes;
}

// Vector math helpers
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt();
    if len > 0.0 {
        scale(v, 1.0 / len)
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_contains_point() {
        let frustum = ViewFrustum::new(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            std::f32::consts::FRAC_PI_2, // 90 degree FOV
            1.0,
            0.1,
            100.0,
        );

        // Point in front of camera
        assert!(frustum.contains_point([0.0, 0.0, -10.0]));
        
        // Point behind camera
        assert!(!frustum.contains_point([0.0, 0.0, 10.0]));
        
        // Point too far
        assert!(!frustum.contains_point([0.0, 0.0, -200.0]));
    }

    #[test]
    fn test_frustum_contains_sphere() {
        let frustum = ViewFrustum::new(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.1,
            100.0,
        );

        // Sphere in front of camera
        assert!(frustum.contains_sphere([0.0, 0.0, -10.0], 5.0));
        
        // Sphere partially visible
        assert!(frustum.contains_sphere([50.0, 0.0, -50.0], 10.0));
        
        // Sphere completely outside
        assert!(!frustum.contains_sphere([200.0, 0.0, -50.0], 5.0));
    }
}