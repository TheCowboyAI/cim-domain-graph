//! Infrastructure layer implementations
//!
//! This module contains concrete implementations of repository traits
//! that bridge the domain layer with external systems like NATS and projections.

mod event_repository_impl;
mod query_repository_impl;
mod unified_repository_impl;

pub use event_repository_impl::AbstractGraphEventRepositoryImpl;
pub use query_repository_impl::AbstractGraphQueryRepositoryImpl;
pub use unified_repository_impl::UnifiedGraphRepositoryImpl;

