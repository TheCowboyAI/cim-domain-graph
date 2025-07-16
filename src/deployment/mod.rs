//! Graph to Nix deployment translation module
//!
//! This module provides functionality to translate visual graph representations
//! of infrastructure into deployable NixOS configurations.

pub mod node_types;
pub mod edge_types;
pub mod translator;
pub mod validation;
pub mod graph_adapter;

pub use node_types::{DeploymentNodeType, ResourceRequirements, HealthCheck, DatabaseEngine, MessageBusType, LoadBalancingStrategy, StorageType, AccessMode};
pub use edge_types::{DeploymentEdgeType, DependencyType};
pub use translator::{GraphToNixTranslator, NixDeploymentSpec, ServiceSpec, StandardTranslator};
pub use validation::{validate_deployment_graph, DeploymentError};