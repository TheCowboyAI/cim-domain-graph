//! Adapter implementations for different graph types

pub mod context_adapter;
pub mod concept_adapter;
pub mod workflow_adapter;
pub mod ipld_adapter;

pub use context_adapter::ContextGraphAdapter;
pub use concept_adapter::ConceptGraphAdapter;
pub use workflow_adapter::WorkflowGraphAdapter;
pub use ipld_adapter::IpldGraphAdapter; 