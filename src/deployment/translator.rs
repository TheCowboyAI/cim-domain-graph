//! Graph to Nix translation implementation

use super::{DeploymentNodeType, DeploymentEdgeType, validation::get_deployment_order, graph_adapter::DeploymentGraphExt};
use crate::aggregate::business_graph::Graph;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for translating deployment graphs to Nix specifications
pub trait GraphToNixTranslator {
    /// Translate a deployment graph to a Nix deployment specification
    fn translate_graph(&self, graph: &Graph) -> Result<NixDeploymentSpec>;
    
    /// Validate that a graph is suitable for deployment
    fn validate_deployment_graph(&self, graph: &Graph) -> Result<()>;
    
    /// Extract service definitions from the graph
    fn extract_services(&self, graph: &Graph) -> Result<Vec<ServiceSpec>>;
    
    /// Extract dependencies from edges
    fn extract_dependencies(&self, graph: &Graph) -> Result<DependencyMap>;
}

/// Complete Nix deployment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixDeploymentSpec {
    pub services: Vec<ServiceSpec>,
    pub databases: Vec<DatabaseSpec>,
    pub agents: Vec<AgentSpec>,
    pub message_buses: Vec<MessageBusSpec>,
    pub load_balancers: Vec<LoadBalancerSpec>,
    pub storage_volumes: Vec<StorageSpec>,
    pub dependencies: DependencyMap,
    pub network_topology: NetworkTopology,
}

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub environment: HashMap<String, String>,
    pub port: Option<u16>,
    pub health_check: Option<HealthCheckSpec>,
    pub resources: Option<ResourceSpec>,
    pub dependencies: Vec<String>,
}

/// Database specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSpec {
    pub name: String,
    pub engine: String,
    pub version: String,
    pub port: u16,
    pub persistent: bool,
    pub backup_schedule: Option<String>,
    pub resources: Option<ResourceSpec>,
}

/// Agent specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    pub capabilities: Vec<String>,
    pub subscriptions: Vec<String>,
    pub nats_url: String,
    pub resources: Option<ResourceSpec>,
}

/// Message bus specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBusSpec {
    pub name: String,
    pub bus_type: String,
    pub cluster_size: usize,
    pub persistence: bool,
    pub ports: Vec<u16>,
}

/// Load balancer specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerSpec {
    pub name: String,
    pub strategy: String,
    pub backends: Vec<BackendSpec>,
    pub health_check_interval: u64,
}

/// Backend specification for load balancers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSpec {
    pub service: String,
    pub port: u16,
    pub weight: Option<u32>,
}

/// Storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    pub name: String,
    pub storage_type: String,
    pub size: String,
    pub mount_paths: Vec<MountSpec>,
}

/// Mount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSpec {
    pub service: String,
    pub path: String,
    pub read_only: bool,
}

/// Health check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    pub endpoint: String,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub retries: u32,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_cores: Option<f32>,
    pub memory_mb: Option<u32>,
    pub disk_gb: Option<u32>,
}

/// Dependency map
pub type DependencyMap = HashMap<String, Vec<String>>;

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub connections: Vec<NetworkConnection>,
    pub exposed_ports: HashMap<String, Vec<u16>>,
}

/// Network connection between services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub from: String,
    pub to: String,
    pub protocol: String,
    pub port: u16,
    pub encrypted: bool,
}

/// Standard implementation of the graph to Nix translator
pub struct StandardTranslator;

impl StandardTranslator {
    /// Create a new standard translator
    pub fn new() -> Self {
        Self
    }
}

impl Default for StandardTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphToNixTranslator for StandardTranslator {
    fn translate_graph(&self, graph: &Graph) -> Result<NixDeploymentSpec> {
        // 1. Validate graph
        self.validate_deployment_graph(graph)?;
        
        // 2. Get deployment order
        let deployment_order = get_deployment_order(graph)?;
        
        // 3. Translate each node
        let mut services = Vec::new();
        let mut databases = Vec::new();
        let mut agents = Vec::new();
        let mut message_buses = Vec::new();
        let mut load_balancers = Vec::new();
        let mut storage_volumes = Vec::new();
        
        for node_id in deployment_order {
            let node = graph.get_node(&node_id)
                .ok_or_else(|| anyhow::anyhow!("Node {} not found", node_id))?;
            
            if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
                match node_type {
                    DeploymentNodeType::Service { .. } => {
                        services.push(self.translate_service_node(&node_type, graph, &node_id)?);
                    }
                    DeploymentNodeType::Database { .. } => {
                        databases.push(self.translate_database_node(&node_type)?);
                    }
                    DeploymentNodeType::Agent { .. } => {
                        agents.push(self.translate_agent_node(&node_type, graph)?);
                    }
                    DeploymentNodeType::MessageBus { .. } => {
                        message_buses.push(self.translate_message_bus_node(&node_type)?);
                    }
                    DeploymentNodeType::LoadBalancer { .. } => {
                        load_balancers.push(self.translate_load_balancer_node(&node_type, graph)?);
                    }
                    DeploymentNodeType::Storage { .. } => {
                        storage_volumes.push(self.translate_storage_node(&node_type, graph, &node_id)?);
                    }
                }
            }
        }
        
        // 4. Extract dependencies and network topology
        let dependencies = self.extract_dependencies(graph)?;
        let network_topology = self.extract_network_topology(graph)?;
        
        Ok(NixDeploymentSpec {
            services,
            databases,
            agents,
            message_buses,
            load_balancers,
            storage_volumes,
            dependencies,
            network_topology,
        })
    }
    
    fn validate_deployment_graph(&self, graph: &Graph) -> Result<()> {
        super::validation::validate_deployment_graph(graph)?;
        Ok(())
    }
    
    fn extract_services(&self, graph: &Graph) -> Result<Vec<ServiceSpec>> {
        let mut services = Vec::new();
        
        for node in graph.get_all_nodes() {
            if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
                if let DeploymentNodeType::Service { .. } = node_type {
                    services.push(self.translate_service_node(&node_type, graph, &node.id)?);
                }
            }
        }
        
        Ok(services)
    }
    
    fn extract_dependencies(&self, graph: &Graph) -> Result<DependencyMap> {
        let mut dependencies = HashMap::new();
        
        for edge in graph.get_all_edges() {
            if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
                if edge_type.is_startup_dependency() {
                    dependencies.entry(edge.from.clone())
                        .or_insert_with(Vec::new)
                        .push(edge.to.clone());
                }
            }
        }
        
        Ok(dependencies)
    }
}

impl StandardTranslator {
    fn translate_service_node(
        &self,
        node: &DeploymentNodeType,
        graph: &Graph,
        node_id: &str,
    ) -> Result<ServiceSpec> {
        if let DeploymentNodeType::Service {
            name,
            command,
            args,
            environment,
            port,
            health_check,
            resources,
        } = node {
            let dependencies = self.get_node_dependencies(graph, node_id)?;
            
            Ok(ServiceSpec {
                name: name.clone(),
                command: command.clone(),
                args: args.clone(),
                environment: environment.clone(),
                port: *port,
                health_check: health_check.as_ref().map(|hc| HealthCheckSpec {
                    endpoint: hc.endpoint.clone(),
                    interval_seconds: hc.interval_seconds,
                    timeout_seconds: hc.timeout_seconds,
                    retries: hc.retries,
                }),
                resources: Some(ResourceSpec {
                    cpu_cores: resources.cpu_cores,
                    memory_mb: resources.memory_mb,
                    disk_gb: resources.disk_gb,
                }),
                dependencies,
            })
        } else {
            Err(anyhow::anyhow!("Not a service node"))
        }
    }
    
    fn translate_database_node(&self, node: &DeploymentNodeType) -> Result<DatabaseSpec> {
        if let DeploymentNodeType::Database {
            name,
            engine,
            version,
            persistent,
            backup_schedule,
            resources,
        } = node {
            let port = match engine {
                super::node_types::DatabaseEngine::PostgreSQL => 5432,
                super::node_types::DatabaseEngine::MySQL => 3306,
                super::node_types::DatabaseEngine::MongoDB => 27017,
                super::node_types::DatabaseEngine::Redis => 6379,
                super::node_types::DatabaseEngine::SQLite => 0,
            };
            
            Ok(DatabaseSpec {
                name: name.clone(),
                engine: format!("{:?}", engine).to_lowercase(),
                version: version.clone(),
                port,
                persistent: *persistent,
                backup_schedule: backup_schedule.clone(),
                resources: Some(ResourceSpec {
                    cpu_cores: resources.cpu_cores,
                    memory_mb: resources.memory_mb,
                    disk_gb: resources.disk_gb,
                }),
            })
        } else {
            Err(anyhow::anyhow!("Not a database node"))
        }
    }
    
    fn translate_agent_node(&self, node: &DeploymentNodeType, graph: &Graph) -> Result<AgentSpec> {
        if let DeploymentNodeType::Agent {
            name,
            capabilities,
            subscriptions,
            resources,
            ..
        } = node {
            // Find NATS connection
            let nats_url = self.find_nats_url(graph)?;
            
            Ok(AgentSpec {
                name: name.clone(),
                capabilities: capabilities.clone(),
                subscriptions: subscriptions.clone(),
                nats_url,
                resources: Some(ResourceSpec {
                    cpu_cores: resources.cpu_cores,
                    memory_mb: resources.memory_mb,
                    disk_gb: resources.disk_gb,
                }),
            })
        } else {
            Err(anyhow::anyhow!("Not an agent node"))
        }
    }
    
    fn translate_message_bus_node(&self, node: &DeploymentNodeType) -> Result<MessageBusSpec> {
        if let DeploymentNodeType::MessageBus {
            name,
            bus_type,
            cluster_size,
            persistence,
            ..
        } = node {
            let ports = match bus_type {
                super::node_types::MessageBusType::NATS => vec![4222, 6222, 8222],
                super::node_types::MessageBusType::Kafka => vec![9092],
                super::node_types::MessageBusType::RabbitMQ => vec![5672, 15672],
                super::node_types::MessageBusType::Redis => vec![6379],
            };
            
            Ok(MessageBusSpec {
                name: name.clone(),
                bus_type: format!("{:?}", bus_type).to_lowercase(),
                cluster_size: *cluster_size,
                persistence: *persistence,
                ports,
            })
        } else {
            Err(anyhow::anyhow!("Not a message bus node"))
        }
    }
    
    fn translate_load_balancer_node(
        &self,
        node: &DeploymentNodeType,
        graph: &Graph,
    ) -> Result<LoadBalancerSpec> {
        if let DeploymentNodeType::LoadBalancer {
            name,
            strategy,
            health_check_interval,
            backends,
        } = node {
            let backend_specs = self.resolve_backends(backends, graph)?;
            
            Ok(LoadBalancerSpec {
                name: name.clone(),
                strategy: format!("{:?}", strategy),
                backends: backend_specs,
                health_check_interval: health_check_interval.as_secs(),
            })
        } else {
            Err(anyhow::anyhow!("Not a load balancer node"))
        }
    }
    
    fn translate_storage_node(
        &self,
        node: &DeploymentNodeType,
        graph: &Graph,
        node_id: &str,
    ) -> Result<StorageSpec> {
        if let DeploymentNodeType::Storage {
            name,
            storage_type,
            size,
            ..
        } = node {
            let mount_specs = self.get_storage_mounts(graph, node_id)?;
            
            Ok(StorageSpec {
                name: name.clone(),
                storage_type: format!("{:?}", storage_type),
                size: size.clone(),
                mount_paths: mount_specs,
            })
        } else {
            Err(anyhow::anyhow!("Not a storage node"))
        }
    }
    
    fn get_node_dependencies(&self, graph: &Graph, node_id: &str) -> Result<Vec<String>> {
        let mut dependencies = Vec::new();
        
        for edge in graph.get_edges_from(node_id) {
            if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
                if edge_type.is_startup_dependency() {
                    dependencies.push(edge.to.clone());
                }
            }
        }
        
        Ok(dependencies)
    }
    
    fn find_nats_url(&self, graph: &Graph) -> Result<String> {
        for node in graph.get_all_nodes() {
            if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
                if let DeploymentNodeType::MessageBus { ref bus_type, .. } = node_type {
                    if matches!(bus_type, super::node_types::MessageBusType::NATS) {
                        return Ok(format!("nats://{}:4222", node_type.name()));
                    }
                }
            }
        }
        
        // Default to localhost if no NATS found
        Ok("nats://localhost:4222".to_string())
    }
    
    fn resolve_backends(&self, backend_names: &[String], graph: &Graph) -> Result<Vec<BackendSpec>> {
        let mut backends = Vec::new();
        
        for backend_name in backend_names {
            if let Some(node) = graph.get_node(backend_name) {
                if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
                    if let DeploymentNodeType::Service { port, .. } = node_type {
                        backends.push(BackendSpec {
                            service: backend_name.clone(),
                            port: port.unwrap_or(80),
                            weight: None,
                        });
                    }
                }
            }
        }
        
        Ok(backends)
    }
    
    fn get_storage_mounts(&self, graph: &Graph, storage_id: &str) -> Result<Vec<MountSpec>> {
        let mut mounts = Vec::new();
        
        for edge in graph.get_edges_to(storage_id) {
            if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
                if let DeploymentEdgeType::MountsVolume { mount_path, read_only } = edge_type {
                    mounts.push(MountSpec {
                        service: edge.from.clone(),
                        path: mount_path,
                        read_only,
                    });
                }
            }
        }
        
        Ok(mounts)
    }
    
    fn extract_network_topology(&self, graph: &Graph) -> Result<NetworkTopology> {
        let mut connections = Vec::new();
        let mut exposed_ports = HashMap::new();
        
        // Extract network connections
        for edge in graph.get_all_edges() {
            if let Ok(edge_type) = serde_json::from_value::<DeploymentEdgeType>(edge.data.clone()) {
                if let DeploymentEdgeType::ConnectsTo { protocol, port, encrypted } = edge_type {
                    connections.push(NetworkConnection {
                        from: edge.from.clone(),
                        to: edge.to.clone(),
                        protocol: format!("{:?}", protocol),
                        port,
                        encrypted,
                    });
                }
            }
        }
        
        // Extract exposed ports
        for node in graph.get_all_nodes() {
            if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(node.data.clone()) {
                let ports = node_type.exposed_ports();
                if !ports.is_empty() {
                    exposed_ports.insert(node.id.clone(), ports);
                }
            }
        }
        
        Ok(NetworkTopology {
            connections,
            exposed_ports,
        })
    }
}

