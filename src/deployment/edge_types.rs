//! Deployment edge types for infrastructure relationships

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents different types of edges in a deployment graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentEdgeType {
    /// Service dependency - target must be running before source
    DependsOn {
        startup_delay: Option<Duration>,
        required: bool,
    },
    /// Network connection between services
    ConnectsTo {
        protocol: NetworkProtocol,
        port: u16,
        encrypted: bool,
    },
    /// Data flow relationship
    DataFlow {
        direction: DataFlowDirection,
        format: DataFormat,
        volume: Option<DataVolume>,
    },
    /// Load balancing relationship
    LoadBalances {
        weight: Option<u32>,
        health_check: bool,
    },
    /// Storage mount relationship
    MountsVolume {
        mount_path: String,
        read_only: bool,
    },
    /// Publishes to message bus topic
    PublishesTo {
        topic: String,
        rate_limit: Option<u32>,
    },
    /// Subscribes to message bus topic
    SubscribesTo {
        topic: String,
        consumer_group: Option<String>,
    },
    /// Manages another service (e.g., agent managing infrastructure)
    Manages {
        permissions: Vec<ManagementPermission>,
    },
}

/// Network protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProtocol {
    HTTP,
    HTTPS,
    TCP,
    UDP,
    GRPC,
    WebSocket,
}

/// Data flow direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFlowDirection {
    Push,
    Pull,
    Bidirectional,
}

/// Data formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    JSON,
    ProtoBuf,
    MessagePack,
    Avro,
    XML,
    Binary,
}

/// Data volume estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataVolume {
    pub messages_per_second: f64,
    pub average_size_bytes: usize,
    pub peak_multiplier: f32,
}

/// Management permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManagementPermission {
    Start,
    Stop,
    Restart,
    Configure,
    Monitor,
    Scale,
    Deploy,
}

/// Dependency types for categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency - target must exist and be healthy
    Hard,
    /// Soft dependency - system can start without it
    Soft,
    /// Runtime dependency - only needed during operation
    Runtime,
}

impl DeploymentEdgeType {
    /// Check if this edge represents a startup dependency
    pub fn is_startup_dependency(&self) -> bool {
        matches!(self, Self::DependsOn { required: true, .. })
    }

    /// Check if this edge requires network connectivity
    pub fn requires_network(&self) -> bool {
        matches!(
            self,
            Self::ConnectsTo { .. }
                | Self::PublishesTo { .. }
                | Self::SubscribesTo { .. }
                | Self::DataFlow { .. }
        )
    }

    /// Get the dependency type for this edge
    pub fn dependency_type(&self) -> DependencyType {
        match self {
            Self::DependsOn { required: true, .. } => DependencyType::Hard,
            Self::DependsOn { required: false, .. } => DependencyType::Soft,
            Self::ConnectsTo { .. } | Self::DataFlow { .. } => DependencyType::Runtime,
            _ => DependencyType::Soft,
        }
    }

    /// Check if this edge requires encryption
    pub fn requires_encryption(&self) -> bool {
        match self {
            Self::ConnectsTo { encrypted, .. } => *encrypted,
            _ => false,
        }
    }

    /// Get network ports required by this edge
    pub fn required_ports(&self) -> Vec<u16> {
        match self {
            Self::ConnectsTo { port, .. } => vec![*port],
            _ => vec![],
        }
    }
}