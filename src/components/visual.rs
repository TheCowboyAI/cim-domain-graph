//! Visual ECS components for graph rendering

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

// Re-export Position3D from value_objects
pub use crate::value_objects::Position3D;

/// Color component
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    /// Create a new color from RGB values
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    
    /// Create a new color from RGBA values
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Size component
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl Size {
    /// Create a new size
    pub fn new(width: f32, height: f32, depth: f32) -> Self {
        Self { width, height, depth }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
        }
    }
}

/// Visual style
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub shape: Shape,
    pub stroke_width: f32,
    pub stroke_color: Color,
    pub fill_color: Color,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shape {
    Circle,
    Rectangle,
    Diamond,
    Hexagon,
    Triangle,
    Custom,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            shape: Shape::Circle,
            stroke_width: 2.0,
            stroke_color: Color::BLACK,
            fill_color: Color::WHITE,
            opacity: 1.0,
        }
    }
}

/// Visibility component
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Visible,
    Hidden,
    Inherited,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

/// 3D transformation
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub struct Transform3D {
    pub position: Position3D,
    pub rotation: Quaternion,
    pub scale: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}


/// Bounding box for spatial queries
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Position3D,
    pub max: Position3D,
}

impl BoundingBox {
    pub fn contains(&self, point: &Position3D) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
} 