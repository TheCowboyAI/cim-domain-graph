//! Graph domain for the Composable Information Machine
//!
//! This is the core composition layer that enables other domains to be composed
//! into graphs without creating dependencies. Other domains do not depend on this
//! domain, but can be composed into graphs through it.

pub mod abstraction;
pub mod aggregate;
pub mod bridge;
pub mod commands;
pub mod components;
pub mod domain_events;
pub mod events;
pub mod handlers;
pub mod infrastructure;
pub mod layout;
pub mod performance;
pub mod plugin;
pub mod projections;
pub mod queries;
pub mod systems;
pub mod value_objects;

// Re-export main types
pub use aggregate::*;
pub use domain_events::*;
pub use events::*;

// Re-export abstraction types
pub use abstraction::{
    ConceptGraphAdapter, ContextGraphAdapter, EdgeData, GraphImplementation, GraphMetadata,
    GraphOperationError, GraphResult, GraphType, IpldGraphAdapter, NodeData, WorkflowGraphAdapter,
};

// Re-export commands and their types
pub use commands::{EdgeCommand, GraphCommand, GraphCommandError, GraphCommandResult, NodeCommand};

// Re-export query types
pub use queries::{
    EdgeInfo, FilterParams, GraphInfo, GraphMetrics, GraphQueryError, GraphQueryHandler,
    GraphQueryHandlerImpl, GraphQueryResult, GraphStructure, NodeInfo, PaginationParams,
};

// Re-export command handlers
pub use handlers::{
    GraphCommandHandler, GraphCommandHandlerImpl, GraphRepository, InMemoryGraphRepository,
};

// Re-export value objects
pub use value_objects::{Color, EdgeType, NodeType, Position2D, Position3D, Style};

// Re-export projections
pub use projections::{GraphProjection, GraphSummaryProjection, NodeListProjection};

// Re-export identifiers that will eventually move here
pub use cim_domain::GraphId;
pub use cim_domain::{EdgeId, NodeId};
