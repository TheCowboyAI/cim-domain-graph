//! Graph projections
//!
//! Projections are optimized read models for graph data that are updated by
//! handling domain events. They provide efficient queries for graph information.

pub mod edge_list;
pub mod graph_summary;
pub mod node_list;

pub use edge_list::*;
pub use graph_summary::*;
pub use node_list::*;

use crate::domain_events::GraphDomainEvent;
use async_trait::async_trait;

/// Trait for graph-specific projections
#[async_trait]
pub trait GraphProjection: Send + Sync {
    /// Handle a graph domain event to update the projection
    async fn handle_graph_event(&mut self, event: GraphDomainEvent) -> Result<(), String>;
}
