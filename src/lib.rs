//! Graph domain for the Composable Information Machine
//!
//! This is the core composition layer that enables other domains to be composed
//! into graphs without creating dependencies. Other domains do not depend on this
//! domain, but can be composed into graphs through it.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod domain_events;

// Re-export main types
pub use aggregate::*;
pub use events::*;
pub use domain_events::*;

// Re-export commands and their types
pub use commands::{
    GraphCommand, NodeCommand, EdgeCommand, 
    GraphCommandResult, GraphCommandError
};

// Re-export query types
pub use queries::{
    GraphQueryHandler, GraphQueryHandlerImpl,
    GraphQueryResult, GraphQueryError,
    GraphInfo, NodeInfo, EdgeInfo, GraphStructure, GraphMetrics,
    PaginationParams, FilterParams
};

// Re-export command handlers
pub use handlers::{
    GraphCommandHandler, GraphCommandHandlerImpl,
    GraphRepository, InMemoryGraphRepository
};

// Re-export value objects
pub use value_objects::{
    NodeType, EdgeType, Position2D, Position3D, Size, Color, Style
};

// Re-export projections
pub use projections::{
    GraphProjection, GraphSummaryProjection, NodeListProjection
};

// Re-export identifiers that will eventually move here
pub use cim_domain::{NodeId, EdgeId};
pub use cim_domain::GraphId;
