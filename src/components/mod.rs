//! ECS Components for the Graph domain
//!
//! This module contains all ECS components used in the graph domain.
//! Components represent the data/state of entities in the system.

pub mod graph;
pub mod node;
pub mod edge;
pub mod visual;
pub mod workflow;
pub mod spatial;

// Re-export commonly used types
pub use graph::{
    GraphEntity, GraphType, GraphStatus, GraphMetadata,
    GraphLayout, LayoutType,
};

pub use node::{
    NodeEntity, NodeType, NodeContent, NodeMetadata,
    NodeStatus, NodeCategory,
};

pub use edge::{
    EdgeEntity, EdgeType, EdgeRelationship, EdgeMetadata,
    EdgeWeight, EdgeDirection,
};

pub use visual::{
    Position3D, Color, Size, Style, Visibility,
    Transform3D, BoundingBox,
};

pub use workflow::{
    WorkflowState, WorkflowStep, WorkflowTransition,
    WorkflowStatus, WorkflowMetadata,
};

pub use spatial::{
    SpatialIndex, GridPosition, QuadrantLocation,
    ProximityGroup, SpatialHash,
};

// Type aliases for common types
pub use crate::{GraphId, NodeId, EdgeId}; 