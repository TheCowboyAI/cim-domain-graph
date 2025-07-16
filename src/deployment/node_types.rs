//! Deployment node types for infrastructure graph representation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Represents different types of nodes in a deployment graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentNodeType {
    /// A service that runs as a systemd unit
    Service {
        name: String,
        command: String,
        args: Vec<String>,
        environment: HashMap<String, String>,
        port: Option<u16>,
        health_check: Option<HealthCheck>,
        resources: ResourceRequirements,
    },
    /// An AI agent that processes tasks
    Agent {
        name: String,
        capabilities: Vec<String>,
        subscriptions: Vec<String>,
        rate_limit: Option<RateLimit>,
        resources: ResourceRequirements,
    },
    /// A database service
    Database {
        name: String,
        engine: DatabaseEngine,
        version: String,
        persistent: bool,
        backup_schedule: Option<String>,
        resources: ResourceRequirements,
    },
    /// A message bus for inter-service communication
    MessageBus {
        name: String,
        bus_type: MessageBusType,
        cluster_size: usize,
        persistence: bool,
        topics: Vec<TopicConfig>,
    },
    /// A load balancer for distributing traffic
    LoadBalancer {
        name: String,
        strategy: LoadBalancingStrategy,
        health_check_interval: Duration,
        backends: Vec<String>, // References to service nodes
    },
    /// Storage volume configuration
    Storage {
        name: String,
        storage_type: StorageType,
        size: String, // "10Gi", "1Ti", etc.
        mount_path: String,
        access_mode: AccessMode,
    },
}

/// Resource requirements for a deployment node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<f32>,
    pub memory_mb: Option<u32>,
    pub disk_gb: Option<u32>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: None,
            memory_mb: None,
            disk_gb: None,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub endpoint: String,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub retries: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

/// Database engine types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseEngine {
    PostgreSQL,
    MySQL,
    MongoDB,
    Redis,
    SQLite,
}

/// Message bus types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBusType {
    NATS,
    Kafka,
    RabbitMQ,
    Redis,
}

/// Topic configuration for message buses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub name: String,
    pub partitions: Option<u32>,
    pub replication_factor: Option<u32>,
    pub retention_hours: Option<u32>,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    IPHash,
    Random,
    Weighted(HashMap<String, u32>),
}

/// Storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    LocalDisk,
    NetworkFS,
    ObjectStore,
    BlockStorage,
}

/// Storage access modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    ReadWriteOnce,
    ReadOnlyMany,
    ReadWriteMany,
}

impl DeploymentNodeType {
    /// Get the name of this deployment node
    pub fn name(&self) -> &str {
        match self {
            Self::Service { name, .. } => name,
            Self::Agent { name, .. } => name,
            Self::Database { name, .. } => name,
            Self::MessageBus { name, .. } => name,
            Self::LoadBalancer { name, .. } => name,
            Self::Storage { name, .. } => name,
        }
    }

    /// Get the resource requirements for this node
    pub fn resources(&self) -> Option<&ResourceRequirements> {
        match self {
            Self::Service { resources, .. } => Some(resources),
            Self::Agent { resources, .. } => Some(resources),
            Self::Database { resources, .. } => Some(resources),
            Self::MessageBus { .. } => None,
            Self::LoadBalancer { .. } => None,
            Self::Storage { .. } => None,
        }
    }

    /// Check if this node type requires persistence
    pub fn requires_persistence(&self) -> bool {
        match self {
            Self::Database { persistent, .. } => *persistent,
            Self::MessageBus { persistence, .. } => *persistence,
            Self::Storage { .. } => true,
            _ => false,
        }
    }

    /// Get exposed ports for this node
    pub fn exposed_ports(&self) -> Vec<u16> {
        match self {
            Self::Service { port, .. } => port.map(|p| vec![p]).unwrap_or_default(),
            Self::Database { engine, .. } => match engine {
                DatabaseEngine::PostgreSQL => vec![5432],
                DatabaseEngine::MySQL => vec![3306],
                DatabaseEngine::MongoDB => vec![27017],
                DatabaseEngine::Redis => vec![6379],
                DatabaseEngine::SQLite => vec![],
            },
            Self::MessageBus { bus_type, .. } => match bus_type {
                MessageBusType::NATS => vec![4222, 6222, 8222],
                MessageBusType::Kafka => vec![9092],
                MessageBusType::RabbitMQ => vec![5672, 15672],
                MessageBusType::Redis => vec![6379],
            },
            _ => vec![],
        }
    }
}