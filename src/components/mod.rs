//! ECS Components for the Graph domain
//!
//! This module contains all ECS components used in the graph domain.
//! Components represent the data/state of entities in the system.

pub mod edge;
pub mod graph;
pub mod node;
pub mod spatial;
pub mod visual;
pub mod workflow;

// Re-export commonly used types
pub use graph::{GraphEntity, GraphLayout, GraphMetadata, GraphStatus, GraphType, LayoutDirection};

pub use node::{NodeCategory, NodeContent, NodeEntity, NodeMetadata, NodeStatus, NodeType};

pub use edge::{
    EdgeColor, EdgeDirection, EdgeEntity, EdgeMetadata, EdgeRelationship, EdgeStyle, EdgeType,
    EdgeWeight,
};

pub use visual::{BoundingBox, Color, Position3D, Size, Style, Transform3D, Visibility};

pub use workflow::{
    RetryPolicy, StepType, WorkflowMetadata, WorkflowState, WorkflowStatus, WorkflowStep,
    WorkflowTransition,
};

pub use spatial::{
    GridPosition, IndexType, ProximityGroup, QuadrantLocation, SpatialHash, SpatialIndex,
};

// Type aliases for common types
pub use crate::{EdgeId, GraphId, NodeId};
