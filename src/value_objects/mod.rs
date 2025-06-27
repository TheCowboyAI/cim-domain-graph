//! Graph value objects
//!
//! Value objects are immutable types that represent concepts in the graph domain.
//! They are compared by value rather than identity and encapsulate domain validation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents different types of nodes in a graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// A task or action node
    Task,
    /// A decision point
    Decision,
    /// A gateway for parallel or conditional flows
    Gateway,
    /// A start node for workflow entry
    Start,
    /// An end node for workflow completion
    End,
    /// A data or document node
    Data,
    /// A service or system integration point
    Service,
    /// An annotation or comment node
    Annotation,
    /// A custom node type
    Custom(String),
}

impl NodeType {
    /// Create a node type from a string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "task" => NodeType::Task,
            "decision" => NodeType::Decision,
            "gateway" => NodeType::Gateway,
            "start" => NodeType::Start,
            "end" => NodeType::End,
            "data" => NodeType::Data,
            "service" => NodeType::Service,
            "annotation" => NodeType::Annotation,
            _ => NodeType::Custom(s.to_string()),
        }
    }

    /// Get the string representation of the node type
    pub fn as_str(&self) -> &str {
        match self {
            NodeType::Task => "task",
            NodeType::Decision => "decision",
            NodeType::Gateway => "gateway",
            NodeType::Start => "start",
            NodeType::End => "end",
            NodeType::Data => "data",
            NodeType::Service => "service",
            NodeType::Annotation => "annotation",
            NodeType::Custom(s) => s,
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Task
    }
}

/// Represents different types of edges in a graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Sequential flow between nodes
    Sequence,
    /// Conditional flow (with condition)
    Conditional(String),
    /// Parallel flow (fork/join)
    Parallel,
    /// Data flow or dependency
    DataFlow,
    /// Association or relationship
    Association,
    /// Composition relationship
    Composition,
    /// Aggregation relationship
    Aggregation,
    /// Custom edge type
    Custom(String),
}

impl EdgeType {
    /// Create an edge type from a string
    pub fn from_str(s: &str) -> Self {
        if s.starts_with("conditional:") {
            let condition = s.strip_prefix("conditional:").unwrap_or("");
            EdgeType::Conditional(condition.to_string())
        } else {
            match s.to_lowercase().as_str() {
                "sequence" => EdgeType::Sequence,
                "parallel" => EdgeType::Parallel,
                "dataflow" | "data_flow" => EdgeType::DataFlow,
                "association" => EdgeType::Association,
                "composition" => EdgeType::Composition,
                "aggregation" => EdgeType::Aggregation,
                _ => EdgeType::Custom(s.to_string()),
            }
        }
    }

    /// Get the string representation of the edge type
    pub fn as_str(&self) -> String {
        match self {
            EdgeType::Sequence => "sequence".to_string(),
            EdgeType::Conditional(condition) => format!("conditional:{condition}"),
            EdgeType::Parallel => "parallel".to_string(),
            EdgeType::DataFlow => "dataflow".to_string(),
            EdgeType::Association => "association".to_string(),
            EdgeType::Composition => "composition".to_string(),
            EdgeType::Aggregation => "aggregation".to_string(),
            EdgeType::Custom(s) => s.clone(),
        }
    }
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for EdgeType {
    fn default() -> Self {
        EdgeType::Sequence
    }
}

/// Represents the position of a node in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl Position2D {
    /// Create a new position
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Get the distance to another position
    pub fn distance_to(&self, other: &Position2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Position2D {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Represents the position of a node in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position3D {
    /// Create a new position
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Get the distance to another position
    pub fn distance_to(&self, other: &Position3D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Convert to 2D position (dropping z coordinate)
    pub fn to_2d(&self) -> Position2D {
        Position2D::new(self.x, self.y)
    }
}

impl Default for Position3D {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl From<Position2D> for Position3D {
    fn from(pos: Position2D) -> Self {
        Self::new(pos.x, pos.y, 0.0)
    }
}

/// Represents the size of a node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    /// Create a new size
    pub fn new(width: f64, height: f64) -> Result<Self, String> {
        if width <= 0.0 || height <= 0.0 {
            return Err("Size dimensions must be positive".to_string());
        }
        Ok(Self { width, height })
    }

    /// Get the area
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 50.0,
        }
    }
}

/// Represents a color value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new opaque color
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Common color constants
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Represents visual style properties for nodes and edges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub fill_color: Color,
    pub border_color: Color,
    pub border_width: f64,
    pub opacity: f64,
}

impl Style {
    /// Create a new style
    pub fn new(fill_color: Color, border_color: Color, border_width: f64, opacity: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&opacity) {
            return Err("Opacity must be between 0.0 and 1.0".to_string());
        }
        if border_width < 0.0 {
            return Err("Border width must be non-negative".to_string());
        }
        Ok(Self {
            fill_color,
            border_color,
            border_width,
            opacity,
        })
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill_color: Color::WHITE,
            border_color: Color::BLACK,
            border_width: 1.0,
            opacity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Coverage
    ///
    /// ```mermaid
    /// graph TD
    ///     VO[Value Objects] --> NT[NodeType]
    ///     VO --> ET[EdgeType]
    ///     VO --> P2[Position2D]
    ///     VO --> P3[Position3D]
    ///     VO --> S[Size]
    ///     VO --> C[Color]
    ///     VO --> ST[Style]
    /// ```

    #[test]
    fn test_node_type_from_string() {
        assert_eq!(NodeType::from_str("task"), NodeType::Task);
        assert_eq!(NodeType::from_str("DECISION"), NodeType::Decision);
        assert_eq!(NodeType::from_str("custom_type"), NodeType::Custom("custom_type".to_string()));
    }

    #[test]
    fn test_node_type_display() {
        assert_eq!(NodeType::Task.to_string(), "task");
        assert_eq!(NodeType::Custom("custom".to_string()).to_string(), "custom");
    }

    #[test]
    fn test_edge_type_conditional() {
        let edge_type = EdgeType::from_str("conditional:x > 0");
        match edge_type {
            EdgeType::Conditional(condition) => assert_eq!(condition, "x > 0"),
            _ => panic!("Expected conditional edge type"),
        }
    }

    #[test]
    fn test_position_distance() {
        let pos1 = Position2D::new(0.0, 0.0);
        let pos2 = Position2D::new(3.0, 4.0);
        
        assert_eq!(pos1.distance_to(&pos2), 5.0);
    }

    #[test]
    fn test_position_3d_to_2d() {
        let pos3d = Position3D::new(1.0, 2.0, 3.0);
        let pos2d = pos3d.to_2d();
        
        assert_eq!(pos2d.x, 1.0);
        assert_eq!(pos2d.y, 2.0);
    }

    #[test]
    fn test_size_validation() {
        assert!(Size::new(10.0, 20.0).is_ok());
        assert!(Size::new(-1.0, 20.0).is_err());
        assert!(Size::new(10.0, 0.0).is_err());
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::RED.g, 0);
        assert_eq!(Color::GREEN.g, 255);
    }

    #[test]
    fn test_style_validation() {
        assert!(Style::new(Color::WHITE, Color::BLACK, 1.0, 0.5).is_ok());
        assert!(Style::new(Color::WHITE, Color::BLACK, 1.0, -0.1).is_err());
        assert!(Style::new(Color::WHITE, Color::BLACK, 1.0, 1.1).is_err());
        assert!(Style::new(Color::WHITE, Color::BLACK, -1.0, 0.5).is_err());
    }

    #[test]
    fn test_serialization() {
        let node_type = NodeType::Custom("test".to_string());
        let serialized = serde_json::to_string(&node_type).unwrap();
        let deserialized: NodeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(node_type, deserialized);

        let position = Position3D::new(1.0, 2.0, 3.0);
        let serialized = serde_json::to_string(&position).unwrap();
        let deserialized: Position3D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(position, deserialized);
    }
}
