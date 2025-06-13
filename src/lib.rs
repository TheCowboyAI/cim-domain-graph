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

// Re-export identifiers that will eventually move here
pub use cim_core_domain::{NodeId, EdgeId};
pub use cim_domain::GraphId;
