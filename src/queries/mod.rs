//! Graph queries
//!
//! Queries provide read-only access to graph data. They operate on projections
//! and read models rather than directly on aggregates.

use crate::value_objects::{Position2D, Position3D};
use crate::{EdgeId, GraphId, NodeId};
use async_trait::async_trait;
use cim_domain::{Query, QueryEnvelope, QueryHandler, QueryResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::{VecDeque, HashSet};

/// Query result type
pub type GraphQueryResult<T> = Result<T, GraphQueryError>;

/// Errors that can occur during graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQueryError {
    /// Graph not found
    GraphNotFound(GraphId),
    /// Node not found
    NodeNotFound(NodeId),
    /// Edge not found
    EdgeNotFound(EdgeId),
    /// Invalid query parameters
    InvalidQuery(String),
    /// Data access error
    DataAccessError(String),
}

impl std::fmt::Display for GraphQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphQueryError::GraphNotFound(id) => write!(f, "Graph not found: {id}"),
            GraphQueryError::NodeNotFound(id) => write!(f, "Node not found: {id}"),
            GraphQueryError::EdgeNotFound(id) => write!(f, "Edge not found: {id}"),
            GraphQueryError::InvalidQuery(msg) => write!(f, "Invalid query: {msg}"),
            GraphQueryError::DataAccessError(msg) => write!(f, "Data access error: {msg}"),
        }
    }
}

impl std::error::Error for GraphQueryError {}

/// Graph information for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInfo {
    pub graph_id: GraphId,
    pub name: String,
    pub description: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node information for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub node_type: String,
    pub position_2d: Option<Position2D>,
    pub position_3d: Option<Position3D>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge information for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Graph structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStructure {
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<EdgeInfo>,
    pub adjacency_list: HashMap<NodeId, Vec<NodeId>>,
}

/// Graph metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub average_degree: f64,
    pub connected_components: usize,
    pub has_cycles: bool,
}

/// Query parameters for pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub offset: usize,
    pub limit: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

/// Query parameters for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParams {
    pub node_types: Option<Vec<String>>,
    pub edge_types: Option<Vec<String>>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    pub name_contains: Option<String>,
}

/// Base query types that implement the Query trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQuery {
    /// Get graph information by ID
    GetGraph { graph_id: GraphId },
    /// Get all graphs with pagination
    GetAllGraphs { pagination: PaginationParams },
    /// Search graphs by name or description
    SearchGraphs {
        query: String,
        pagination: PaginationParams,
    },
    /// Filter graphs by criteria
    FilterGraphs {
        filter: FilterParams,
        pagination: PaginationParams,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeQuery {
    /// Get node information by ID
    GetNode { node_id: NodeId },
    /// Get all nodes in a graph
    GetNodesInGraph { graph_id: GraphId },
    /// Get nodes by type
    GetNodesByType {
        graph_id: GraphId,
        node_type: String,
    },
    /// Find nodes within a radius of a position
    FindNodesNearPosition {
        graph_id: GraphId,
        center: Position2D,
        radius: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeQuery {
    /// Get edge information by ID
    GetEdge { edge_id: EdgeId },
    /// Get all edges in a graph
    GetEdgesInGraph { graph_id: GraphId },
    /// Get edges by type
    GetEdgesByType {
        graph_id: GraphId,
        edge_type: String,
    },
    /// Get edges connected to a node
    GetNodeEdges { node_id: NodeId },
    /// Get incoming edges for a node
    GetIncomingEdges { node_id: NodeId },
    /// Get outgoing edges for a node
    GetOutgoingEdges { node_id: NodeId },
}

/// Trait for graph query operations
#[async_trait]
pub trait GraphQueryHandler: Send + Sync {
    // Graph-level queries

    /// Get graph information by ID
    async fn get_graph(&self, graph_id: GraphId) -> GraphQueryResult<GraphInfo>;

    /// Get all graphs with pagination
    async fn get_all_graphs(
        &self,
        pagination: PaginationParams,
    ) -> GraphQueryResult<Vec<GraphInfo>>;

    /// Search graphs by name or description
    async fn search_graphs(
        &self,
        query: &str,
        pagination: PaginationParams,
    ) -> GraphQueryResult<Vec<GraphInfo>>;

    /// Filter graphs by criteria
    async fn filter_graphs(
        &self,
        filter: FilterParams,
        pagination: PaginationParams,
    ) -> GraphQueryResult<Vec<GraphInfo>>;

    // Node-level queries

    /// Get node information by ID
    async fn get_node(&self, node_id: NodeId) -> GraphQueryResult<NodeInfo>;

    /// Get all nodes in a graph
    async fn get_nodes_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>>;

    /// Get nodes by type
    async fn get_nodes_by_type(
        &self,
        graph_id: GraphId,
        node_type: &str,
    ) -> GraphQueryResult<Vec<NodeInfo>>;

    /// Find nodes within a radius of a position
    async fn find_nodes_near_position(
        &self,
        graph_id: GraphId,
        center: Position2D,
        radius: f64,
    ) -> GraphQueryResult<Vec<NodeInfo>>;

    // Edge-level queries

    /// Get edge information by ID
    async fn get_edge(&self, edge_id: EdgeId) -> GraphQueryResult<EdgeInfo>;

    /// Get all edges in a graph
    async fn get_edges_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<EdgeInfo>>;

    /// Get edges by type
    async fn get_edges_by_type(
        &self,
        graph_id: GraphId,
        edge_type: &str,
    ) -> GraphQueryResult<Vec<EdgeInfo>>;

    /// Get edges connected to a node
    async fn get_node_edges(&self, node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>>;

    /// Get incoming edges for a node
    async fn get_incoming_edges(&self, node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>>;

    /// Get outgoing edges for a node
    async fn get_outgoing_edges(&self, node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>>;

    // Structure and analysis queries

    /// Get the complete graph structure
    async fn get_graph_structure(&self, graph_id: GraphId) -> GraphQueryResult<GraphStructure>;

    /// Get graph metrics and analysis
    async fn get_graph_metrics(&self, graph_id: GraphId) -> GraphQueryResult<GraphMetrics>;

    /// Find connected components in a graph
    async fn find_connected_components(
        &self,
        graph_id: GraphId,
    ) -> GraphQueryResult<Vec<Vec<NodeId>>>;

    /// Find shortest path between two nodes
    async fn find_shortest_path(
        &self,
        graph_id: GraphId,
        source: NodeId,
        target: NodeId,
    ) -> GraphQueryResult<Option<Vec<NodeId>>>;

    /// Check if graph contains cycles
    async fn has_cycles(&self, graph_id: GraphId) -> GraphQueryResult<bool>;

    /// Find nodes with no incoming edges (sources)
    async fn find_source_nodes(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>>;

    /// Find nodes with no outgoing edges (sinks)
    async fn find_sink_nodes(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>>;
}

/// Implementation of graph query handler with CQRS support
pub struct GraphQueryHandlerImpl {
    graph_summary_projection: crate::projections::GraphSummaryProjection,
    node_list_projection: crate::projections::NodeListProjection,
    edge_list_projection: crate::projections::EdgeListProjection,
}

impl Default for GraphQueryHandlerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphQueryHandlerImpl {
    /// Create a new graph query handler
    pub fn new() -> Self {
        Self {
            graph_summary_projection: crate::projections::GraphSummaryProjection::new(),
            node_list_projection: crate::projections::NodeListProjection::new(),
            edge_list_projection: crate::projections::EdgeListProjection::new(),
        }
    }

    /// Create with existing projections
    pub fn with_projections(
        graph_summary_projection: crate::projections::GraphSummaryProjection,
        node_list_projection: crate::projections::NodeListProjection,
        edge_list_projection: crate::projections::EdgeListProjection,
    ) -> Self {
        Self {
            graph_summary_projection,
            node_list_projection,
            edge_list_projection,
        }
    }
}

// Implement Query trait for all query types
impl Query for GraphQuery {}
impl Query for NodeQuery {}
impl Query for EdgeQuery {}

// Implement QueryHandler for GraphQuery
impl QueryHandler<GraphQuery> for GraphQueryHandlerImpl {
    fn handle(&self, envelope: QueryEnvelope<GraphQuery>) -> QueryResponse {
        let _query_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();

        // Process the query synchronously (blocking on async)
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async {
            match &envelope.query {
                GraphQuery::GetGraph { graph_id } => {
                    self.get_graph(*graph_id).await.map(|info| {
                        // TODO: Publish result to event stream with correlation
                        serde_json::to_value(info).unwrap()
                    })
                }
                GraphQuery::GetAllGraphs { pagination } => {
                    self.get_all_graphs(pagination.clone()).await.map(|infos| {
                        // TODO: Publish result to event stream with correlation
                        serde_json::to_value(infos).unwrap()
                    })
                }
                GraphQuery::SearchGraphs { query, pagination } => {
                    self.search_graphs(query, pagination.clone())
                        .await
                        .map(|infos| {
                            // TODO: Publish result to event stream with correlation
                            serde_json::to_value(infos).unwrap()
                        })
                }
                GraphQuery::FilterGraphs { filter, pagination } => {
                    self.filter_graphs(filter.clone(), pagination.clone())
                        .await
                        .map(|infos| {
                            // TODO: Publish result to event stream with correlation
                            serde_json::to_value(infos).unwrap()
                        })
                }
            }
        });

        match result {
            Ok(value) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: value,
            },
            Err(error) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: serde_json::json!({
                    "error": error.to_string()
                }),
            },
        }
    }
}

// Implement QueryHandler for NodeQuery
impl QueryHandler<NodeQuery> for GraphQueryHandlerImpl {
    fn handle(&self, envelope: QueryEnvelope<NodeQuery>) -> QueryResponse {
        let _query_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();

        // Process the query synchronously (blocking on async)
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async {
            match &envelope.query {
                NodeQuery::GetNode { node_id } => {
                    self.get_node(*node_id).await.map(|info| {
                        // TODO: Publish result to event stream with correlation
                        serde_json::to_value(info).unwrap()
                    })
                }
                NodeQuery::GetNodesInGraph { graph_id } => {
                    self.get_nodes_in_graph(*graph_id).await.map(|infos| {
                        // TODO: Publish result to event stream with correlation
                        serde_json::to_value(infos).unwrap()
                    })
                }
                NodeQuery::GetNodesByType {
                    graph_id,
                    node_type,
                } => {
                    self.get_nodes_by_type(*graph_id, node_type)
                        .await
                        .map(|infos| {
                            // TODO: Publish result to event stream with correlation
                            serde_json::to_value(infos).unwrap()
                        })
                }
                NodeQuery::FindNodesNearPosition {
                    graph_id,
                    center,
                    radius,
                } => {
                    self.find_nodes_near_position(*graph_id, *center, *radius)
                        .await
                        .map(|infos| {
                            // TODO: Publish result to event stream with correlation
                            serde_json::to_value(infos).unwrap()
                        })
                }
            }
        });

        match result {
            Ok(value) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: value,
            },
            Err(error) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: serde_json::json!({
                    "error": error.to_string()
                }),
            },
        }
    }
}

// Implement QueryHandler for EdgeQuery
impl QueryHandler<EdgeQuery> for GraphQueryHandlerImpl {
    fn handle(&self, envelope: QueryEnvelope<EdgeQuery>) -> QueryResponse {
        let _query_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();

        // Process the query synchronously (blocking on async)
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async {
            match &envelope.query {
                EdgeQuery::GetEdge { edge_id } => {
                    self.get_edge(*edge_id).await.map(|info| {
                        serde_json::to_value(info).unwrap()
                    })
                }
                EdgeQuery::GetEdgesInGraph { graph_id } => {
                    self.get_edges_in_graph(*graph_id).await.map(|infos| {
                        serde_json::to_value(infos).unwrap()
                    })
                }
                EdgeQuery::GetEdgesByType {
                    graph_id,
                    edge_type,
                } => {
                    self.get_edges_by_type(*graph_id, edge_type)
                        .await
                        .map(|infos| {
                            serde_json::to_value(infos).unwrap()
                        })
                }
                EdgeQuery::GetNodeEdges { node_id } => {
                    self.get_node_edges(*node_id).await.map(|infos| {
                        serde_json::to_value(infos).unwrap()
                    })
                }
                EdgeQuery::GetIncomingEdges { node_id } => {
                    self.get_incoming_edges(*node_id).await.map(|infos| {
                        serde_json::to_value(infos).unwrap()
                    })
                }
                EdgeQuery::GetOutgoingEdges { node_id } => {
                    self.get_outgoing_edges(*node_id).await.map(|infos| {
                        serde_json::to_value(infos).unwrap()
                    })
                }
            }
        });

        match result {
            Ok(value) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: value,
            },
            Err(error) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: serde_json::json!({
                    "error": error.to_string()
                }),
            },
        }
    }
}

#[async_trait]
impl GraphQueryHandler for GraphQueryHandlerImpl {
    async fn get_graph(&self, graph_id: GraphId) -> GraphQueryResult<GraphInfo> {
        match self.graph_summary_projection.get_summary(&graph_id) {
            Some(summary) => Ok(GraphInfo {
                graph_id: summary.graph_id,
                name: summary.name.clone(),
                description: summary.description.clone(),
                node_count: summary.node_count,
                edge_count: summary.edge_count,
                created_at: summary.created_at,
                last_modified: summary.last_modified,
                metadata: summary.metadata.clone(),
            }),
            None => Err(GraphQueryError::GraphNotFound(graph_id)),
        }
    }

    async fn get_all_graphs(
        &self,
        pagination: PaginationParams,
    ) -> GraphQueryResult<Vec<GraphInfo>> {
        let summaries = self
            .graph_summary_projection
            .get_summaries_paginated(pagination.offset, pagination.limit);

        let graph_infos = summaries
            .into_iter()
            .map(|summary| GraphInfo {
                graph_id: summary.graph_id,
                name: summary.name.clone(),
                description: summary.description.clone(),
                node_count: summary.node_count,
                edge_count: summary.edge_count,
                created_at: summary.created_at,
                last_modified: summary.last_modified,
                metadata: summary.metadata.clone(),
            })
            .collect();

        Ok(graph_infos)
    }

    async fn search_graphs(
        &self,
        query: &str,
        pagination: PaginationParams,
    ) -> GraphQueryResult<Vec<GraphInfo>> {
        let query_lower = query.to_lowercase();
        let all_summaries = self.graph_summary_projection.get_all_summaries();

        let matching_summaries: Vec<_> = all_summaries
            .into_iter()
            .filter(|summary| {
                summary.name.to_lowercase().contains(&query_lower)
                    || summary.description.to_lowercase().contains(&query_lower)
            })
            .skip(pagination.offset)
            .take(pagination.limit)
            .collect();

        let graph_infos = matching_summaries
            .into_iter()
            .map(|summary| GraphInfo {
                graph_id: summary.graph_id,
                name: summary.name.clone(),
                description: summary.description.clone(),
                node_count: summary.node_count,
                edge_count: summary.edge_count,
                created_at: summary.created_at,
                last_modified: summary.last_modified,
                metadata: summary.metadata.clone(),
            })
            .collect();

        Ok(graph_infos)
    }

    async fn filter_graphs(
        &self,
        filter: FilterParams,
        pagination: PaginationParams,
    ) -> GraphQueryResult<Vec<GraphInfo>> {
        let all_summaries = self.graph_summary_projection.get_all_summaries();

        let filtered_summaries: Vec<_> = all_summaries
            .into_iter()
            .filter(|summary| {
                // Filter by creation date range
                if let Some(after) = filter.created_after {
                    if summary.created_at < after {
                        return false;
                    }
                }

                if let Some(before) = filter.created_before {
                    if summary.created_at > before {
                        return false;
                    }
                }

                // Filter by name contains
                if let Some(name_filter) = &filter.name_contains {
                    if !summary
                        .name
                        .to_lowercase()
                        .contains(&name_filter.to_lowercase())
                    {
                        return false;
                    }
                }

                true
            })
            .skip(pagination.offset)
            .take(pagination.limit)
            .collect();

        let graph_infos = filtered_summaries
            .into_iter()
            .map(|summary| GraphInfo {
                graph_id: summary.graph_id,
                name: summary.name.clone(),
                description: summary.description.clone(),
                node_count: summary.node_count,
                edge_count: summary.edge_count,
                created_at: summary.created_at,
                last_modified: summary.last_modified,
                metadata: summary.metadata.clone(),
            })
            .collect();

        Ok(graph_infos)
    }

    async fn get_node(&self, node_id: NodeId) -> GraphQueryResult<NodeInfo> {
        match self.node_list_projection.get_node(&node_id) {
            Some(node_info) => Ok(NodeInfo {
                node_id: node_info.node_id,
                graph_id: node_info.graph_id,
                node_type: node_info.node_type.clone(),
                position_2d: None, // TODO: Add position tracking to projections
                position_3d: None, // TODO: Add position tracking to projections
                metadata: node_info.metadata.clone(),
            }),
            None => Err(GraphQueryError::NodeNotFound(node_id)),
        }
    }

    async fn get_nodes_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        let node_infos = self
            .node_list_projection
            .get_nodes_by_graph(&graph_id)
            .into_iter()
            .map(|node_info| NodeInfo {
                node_id: node_info.node_id,
                graph_id: node_info.graph_id,
                node_type: node_info.node_type.clone(),
                position_2d: None, // TODO: Add position tracking to projections
                position_3d: None, // TODO: Add position tracking to projections
                metadata: node_info.metadata.clone(),
            })
            .collect();

        Ok(node_infos)
    }

    async fn get_nodes_by_type(
        &self,
        graph_id: GraphId,
        node_type: &str,
    ) -> GraphQueryResult<Vec<NodeInfo>> {
        let all_typed_nodes = self.node_list_projection.get_nodes_by_type(node_type);

        let filtered_nodes: Vec<_> = all_typed_nodes
            .into_iter()
            .filter(|node_info| node_info.graph_id == graph_id)
            .map(|node_info| NodeInfo {
                node_id: node_info.node_id,
                graph_id: node_info.graph_id,
                node_type: node_info.node_type.clone(),
                position_2d: None, // TODO: Add position tracking to projections
                position_3d: None, // TODO: Add position tracking to projections
                metadata: node_info.metadata.clone(),
            })
            .collect();

        Ok(filtered_nodes)
    }

    async fn find_nodes_near_position(
        &self,
        graph_id: GraphId,
        center: Position2D,
        radius: f64,
    ) -> GraphQueryResult<Vec<NodeInfo>> {
        // Get all nodes in the graph
        let nodes = self.node_list_projection.get_nodes_by_graph(&graph_id);

        // Filter nodes by distance from center
        let nearby_nodes: Vec<NodeInfo> = nodes
            .into_iter()
            .filter_map(|node| {
                // Try to get position from metadata
                let pos_x = node.metadata.get("position_x")?.as_f64()?;
                let pos_y = node.metadata.get("position_y")?.as_f64()?;
                
                // Calculate distance
                let dx = pos_x - center.x;
                let dy = pos_y - center.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance <= radius {
                    Some(NodeInfo {
                        node_id: node.node_id,
                        graph_id: node.graph_id,
                        node_type: node.node_type.clone(),
                        position_2d: Some(Position2D { x: pos_x, y: pos_y }),
                        position_3d: None,
                        metadata: node.metadata.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(nearby_nodes)
    }

    async fn get_edge(&self, edge_id: EdgeId) -> GraphQueryResult<EdgeInfo> {
        match self.edge_list_projection.get_edge(&edge_id) {
            Some(edge) => Ok(EdgeInfo {
                edge_id: edge.edge_id,
                graph_id: edge.graph_id,
                source_id: edge.source_id,
                target_id: edge.target_id,
                edge_type: edge.edge_type.clone(),
                metadata: edge.metadata.clone(),
            }),
            None => Err(GraphQueryError::EdgeNotFound(edge_id)),
        }
    }

    async fn get_edges_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<EdgeInfo>> {
        let edges = self.edge_list_projection.get_edges_by_graph(&graph_id);
        
        let edge_infos = edges
            .into_iter()
            .map(|edge| EdgeInfo {
                edge_id: edge.edge_id,
                graph_id: edge.graph_id,
                source_id: edge.source_id,
                target_id: edge.target_id,
                edge_type: edge.edge_type.clone(),
                metadata: edge.metadata.clone(),
            })
            .collect();

        Ok(edge_infos)
    }

    async fn get_edges_by_type(
        &self,
        graph_id: GraphId,
        edge_type: &str,
    ) -> GraphQueryResult<Vec<EdgeInfo>> {
        let all_edges = self.edge_list_projection.get_edges_by_type(edge_type);
        
        let edge_infos = all_edges
            .into_iter()
            .filter(|edge| edge.graph_id == graph_id)
            .map(|edge| EdgeInfo {
                edge_id: edge.edge_id,
                graph_id: edge.graph_id,
                source_id: edge.source_id,
                target_id: edge.target_id,
                edge_type: edge.edge_type.clone(),
                metadata: edge.metadata.clone(),
            })
            .collect();

        Ok(edge_infos)
    }

    async fn get_node_edges(&self, node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        let edges = self.edge_list_projection.get_edges_for_node(&node_id);
        
        let edge_infos = edges
            .into_iter()
            .map(|edge| EdgeInfo {
                edge_id: edge.edge_id,
                graph_id: edge.graph_id,
                source_id: edge.source_id,
                target_id: edge.target_id,
                edge_type: edge.edge_type.clone(),
                metadata: edge.metadata.clone(),
            })
            .collect();

        Ok(edge_infos)
    }

    async fn get_incoming_edges(&self, node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        let edges = self.edge_list_projection.get_incoming_edges(&node_id);
        
        let edge_infos = edges
            .into_iter()
            .map(|edge| EdgeInfo {
                edge_id: edge.edge_id,
                graph_id: edge.graph_id,
                source_id: edge.source_id,
                target_id: edge.target_id,
                edge_type: edge.edge_type.clone(),
                metadata: edge.metadata.clone(),
            })
            .collect();

        Ok(edge_infos)
    }

    async fn get_outgoing_edges(&self, node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        let edges = self.edge_list_projection.get_outgoing_edges(&node_id);
        
        let edge_infos = edges
            .into_iter()
            .map(|edge| EdgeInfo {
                edge_id: edge.edge_id,
                graph_id: edge.graph_id,
                source_id: edge.source_id,
                target_id: edge.target_id,
                edge_type: edge.edge_type.clone(),
                metadata: edge.metadata.clone(),
            })
            .collect();

        Ok(edge_infos)
    }

    async fn get_graph_structure(&self, graph_id: GraphId) -> GraphQueryResult<GraphStructure> {
        // Get all nodes in the graph
        let nodes = self.get_nodes_in_graph(graph_id).await?;

        // Get all edges in the graph
        let edges = self.get_edges_in_graph(graph_id).await?;

        // Build adjacency list using edge projection
        let adjacency_list = self.edge_list_projection.get_adjacency_list(&graph_id);

        Ok(GraphStructure {
            nodes,
            edges,
            adjacency_list,
        })
    }

    async fn get_graph_metrics(&self, graph_id: GraphId) -> GraphQueryResult<GraphMetrics> {
        // First check if the graph exists
        if self.graph_summary_projection.get_summary(&graph_id).is_none() {
            return Err(GraphQueryError::GraphNotFound(graph_id));
        }

        // Get basic counts
        let node_count = self.node_list_projection.get_node_count_for_graph(&graph_id);
        let edge_count = self.edge_list_projection.get_edge_count_for_graph(&graph_id);

        // Calculate density (for directed graph)
        // Density = edges / (nodes * (nodes - 1))
        let density = if node_count > 1 {
            edge_count as f64 / (node_count as f64 * (node_count - 1) as f64)
        } else {
            0.0
        };

        // Calculate average degree
        // For directed graph: average out-degree
        let average_degree = if node_count > 0 {
            (edge_count as f64) / (node_count as f64)
        } else {
            0.0
        };

        // Check for cycles
        let has_cycles = self.has_cycles(graph_id).await.unwrap_or(false);

        // Count connected components
        let components = self.find_connected_components(graph_id).await.unwrap_or_default();
        let connected_components = components.len();

        Ok(GraphMetrics {
            node_count,
            edge_count,
            density,
            average_degree,
            connected_components,
            has_cycles,
        })
    }

    async fn find_connected_components(
        &self,
        graph_id: GraphId,
    ) -> GraphQueryResult<Vec<Vec<NodeId>>> {
        use std::collections::HashSet;

        // Get all edges to build undirected adjacency list
        let edges = self.edge_list_projection.get_edges_by_graph(&graph_id);
        let mut undirected_adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        // Build undirected adjacency list
        for edge in edges {
            undirected_adj
                .entry(edge.source_id)
                .or_default()
                .push(edge.target_id);
            undirected_adj
                .entry(edge.target_id)
                .or_default()
                .push(edge.source_id);
        }

        // Get all nodes in the graph
        let nodes = self.node_list_projection.get_nodes_by_graph(&graph_id);
        let all_nodes: HashSet<NodeId> = nodes.iter().map(|n| n.node_id).collect();

        // Track visited nodes
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        // Helper function for DFS
        fn dfs_component(
            node: NodeId,
            undirected_adj: &HashMap<NodeId, Vec<NodeId>>,
            visited: &mut HashSet<NodeId>,
            component: &mut Vec<NodeId>,
        ) {
            visited.insert(node);
            component.push(node);

            if let Some(neighbors) = undirected_adj.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        dfs_component(neighbor, undirected_adj, visited, component);
                    }
                }
            }
        }

        // Find all connected components
        for node in all_nodes {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                dfs_component(node, &undirected_adj, &mut visited, &mut component);
                components.push(component);
            }
        }

        Ok(components)
    }

    async fn find_shortest_path(
        &self,
        graph_id: GraphId,
        source: NodeId,
        target: NodeId,
    ) -> GraphQueryResult<Option<Vec<NodeId>>> {
        // Get adjacency list for the graph
        let adjacency = self.edge_list_projection.get_adjacency_list(&graph_id);

        // Check if source and target exist in the graph
        let nodes = self.node_list_projection.get_nodes_by_graph(&graph_id);
        let node_ids: HashSet<NodeId> = nodes.iter().map(|n| n.node_id).collect();
        
        if !node_ids.contains(&source) || !node_ids.contains(&target) {
            return Ok(None);
        }

        // BFS to find shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = HashMap::new();

        queue.push_back(source);
        visited.insert(source);

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = target;
                
                while node != source {
                    path.push(node);
                    node = *parent_map.get(&node).unwrap();
                }
                path.push(source);
                path.reverse();
                
                return Ok(Some(path));
            }

            // Visit neighbors
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent_map.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // No path found
        Ok(None)
    }

    async fn has_cycles(&self, graph_id: GraphId) -> GraphQueryResult<bool> {
        use std::collections::HashSet;

        // Get adjacency list for the graph
        let adjacency = self.edge_list_projection.get_adjacency_list(&graph_id);

        // Get all nodes in the graph
        let nodes = self.node_list_projection.get_nodes_by_graph(&graph_id);
        let all_nodes: HashSet<NodeId> = nodes.iter().map(|n| n.node_id).collect();

        // Track visited nodes and nodes in current path
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        // Helper function for DFS
        fn dfs_has_cycle(
            node: NodeId,
            adjacency: &HashMap<NodeId, Vec<NodeId>>,
            visited: &mut HashSet<NodeId>,
            rec_stack: &mut HashSet<NodeId>,
        ) -> bool {
            visited.insert(node);
            rec_stack.insert(node);

            if let Some(neighbors) = adjacency.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        if dfs_has_cycle(neighbor, adjacency, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(&neighbor) {
                        // Found a back edge (cycle)
                        return true;
                    }
                }
            }

            rec_stack.remove(&node);
            false
        }

        // Check for cycles starting from each unvisited node
        for node in all_nodes {
            if !visited.contains(&node)
                && dfs_has_cycle(node, &adjacency, &mut visited, &mut rec_stack) {
                    return Ok(true);
                }
        }

        Ok(false)
    }

    async fn find_source_nodes(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // Get all nodes in the graph
        let all_nodes = self.node_list_projection.get_nodes_by_graph(&graph_id);
        
        // Get all edges in the graph to find nodes with incoming edges
        let edges = self.edge_list_projection.get_edges_by_graph(&graph_id);
        
        // Create a set of nodes that have incoming edges
        let mut nodes_with_incoming: HashSet<NodeId> = HashSet::new();
        for edge in edges {
            nodes_with_incoming.insert(edge.target_id);
        }
        
        // Filter nodes that have no incoming edges (source nodes)
        let source_nodes: Vec<NodeInfo> = all_nodes
            .into_iter()
            .filter(|node| !nodes_with_incoming.contains(&node.node_id))
            .map(|node_info| NodeInfo {
                node_id: node_info.node_id,
                graph_id: node_info.graph_id,
                node_type: node_info.node_type.clone(),
                position_2d: None,
                position_3d: None,
                metadata: node_info.metadata.clone(),
            })
            .collect();
        
        Ok(source_nodes)
    }

    async fn find_sink_nodes(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // Get all nodes in the graph
        let all_nodes = self.node_list_projection.get_nodes_by_graph(&graph_id);
        
        // Get all edges in the graph to find nodes with outgoing edges
        let edges = self.edge_list_projection.get_edges_by_graph(&graph_id);
        
        // Create a set of nodes that have outgoing edges
        let mut nodes_with_outgoing: HashSet<NodeId> = HashSet::new();
        for edge in edges {
            nodes_with_outgoing.insert(edge.source_id);
        }
        
        // Filter nodes that have no outgoing edges (sink nodes)
        let sink_nodes: Vec<NodeInfo> = all_nodes
            .into_iter()
            .filter(|node| !nodes_with_outgoing.contains(&node.node_id))
            .map(|node_info| NodeInfo {
                node_id: node_info.node_id,
                graph_id: node_info.graph_id,
                node_type: node_info.node_type.clone(),
                position_2d: None,
                position_3d: None,
                metadata: node_info.metadata.clone(),
            })
            .collect();
        
        Ok(sink_nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EdgeAdded, GraphCreated, NodeAdded};
    use crate::components::EdgeRelationship;
    use crate::domain_events::GraphDomainEvent;
    use crate::projections::GraphProjection;
    use crate::value_objects::Position3D;
    use chrono::Utc;

    /// Test Coverage
    ///
    /// ```mermaid
    /// graph TD
    ///     Q[Queries] --> GQ[Graph Queries]
    ///     Q --> NQ[Node Queries]
    ///     Q --> EQ[Edge Queries]
    ///     Q --> SQ[Structure Queries]
    ///     GQ --> H[Query Handler]
    ///     NQ --> H
    ///     EQ --> H
    ///     SQ --> H
    /// ```

    #[test]
    fn test_query_handler_creation() {
        let handler = GraphQueryHandlerImpl::new();
        // Handler should be created with empty projections
        // This is just a basic creation test
        let _ = handler; // Silence unused variable warning
    }

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.offset, 0);
        assert_eq!(params.limit, 50);
    }

    #[test]
    fn test_graph_query_error_display() {
        let graph_id = GraphId::new();
        let error = GraphQueryError::GraphNotFound(graph_id);

        let display = format!("{error}");
        assert!(display.contains("Graph not found"));
        assert!(display.contains(&graph_id.to_string()));
    }

    #[test]
    fn test_query_types_serialization() {
        let graph_info = GraphInfo {
            graph_id: GraphId::new(),
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            node_count: 5,
            edge_count: 4,
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&graph_info).unwrap();
        let deserialized: GraphInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(graph_info.name, deserialized.name);
        assert_eq!(graph_info.node_count, deserialized.node_count);
    }

    #[test]
    fn test_filter_params() {
        let filter = FilterParams {
            node_types: Some(vec!["task".to_string(), "decision".to_string()]),
            edge_types: Some(vec!["sequence".to_string()]),
            created_after: Some(chrono::Utc::now()),
            created_before: None,
            name_contains: Some("test".to_string()),
        };

        assert_eq!(filter.node_types.as_ref().unwrap().len(), 2);
        assert!(filter.name_contains.as_ref().unwrap().contains("test"));
    }

    #[tokio::test]
    async fn test_graph_queries_with_data() {
        use crate::domain_events::GraphDomainEvent;
        use crate::events::{GraphCreated, NodeAdded};
        use crate::projections::{GraphProjection, GraphSummaryProjection, NodeListProjection};
        use chrono::Utc;

        // Create projections with test data
        let mut graph_summary = GraphSummaryProjection::new();
        let mut node_list = NodeListProjection::new();

        let graph_id = GraphId::new();
        let node_id = NodeId::new();
        let now = Utc::now();

        // Add test graph
        let graph_event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id,
            name: "Test Graph".to_string(),
            description: "A test graph for queries".to_string(),
            graph_type: None,
            metadata: std::collections::HashMap::new(),
            created_at: now,
        });

        graph_summary
            .handle_graph_event(graph_event.clone())
            .await
            .unwrap();

        // Add test node
        let mut node_metadata = std::collections::HashMap::new();
        node_metadata.insert(
            "name".to_string(),
            serde_json::Value::String("Test Node".to_string()),
        );

        let node_event = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id,
            position: crate::value_objects::Position3D::default(),
            node_type: "TestType".to_string(),
            metadata: node_metadata,
        });

        // Handle node event in both projections
        graph_summary
            .handle_graph_event(node_event.clone())
            .await
            .unwrap();
        node_list.handle_graph_event(node_event).await.unwrap();

        // Create handler with test data
        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, crate::projections::EdgeListProjection::new());

        // Test get_graph
        let graph_info = handler.get_graph(graph_id).await.unwrap();
        assert_eq!(graph_info.name, "Test Graph");
        assert_eq!(graph_info.node_count, 1);
        assert_eq!(graph_info.edge_count, 0);

        // Test get_all_graphs
        let all_graphs = handler
            .get_all_graphs(PaginationParams::default())
            .await
            .unwrap();
        assert_eq!(all_graphs.len(), 1);
        assert_eq!(all_graphs[0].graph_id, graph_id);

        // Test search_graphs
        let search_results = handler
            .search_graphs("test", PaginationParams::default())
            .await
            .unwrap();
        assert_eq!(search_results.len(), 1);

        let no_results = handler
            .search_graphs("nonexistent", PaginationParams::default())
            .await
            .unwrap();
        assert_eq!(no_results.len(), 0);

        // Test get_node
        let node_info = handler.get_node(node_id).await.unwrap();
        assert_eq!(node_info.node_id, node_id);
        assert_eq!(node_info.graph_id, graph_id);
        assert_eq!(node_info.node_type, "TestType");

        // Test get_nodes_in_graph
        let nodes_in_graph = handler.get_nodes_in_graph(graph_id).await.unwrap();
        assert_eq!(nodes_in_graph.len(), 1);

        // Test get_nodes_by_type
        let typed_nodes = handler
            .get_nodes_by_type(graph_id, "TestType")
            .await
            .unwrap();
        assert_eq!(typed_nodes.len(), 1);

        let no_typed_nodes = handler
            .get_nodes_by_type(graph_id, "NonExistentType")
            .await
            .unwrap();
        assert_eq!(no_typed_nodes.len(), 0);

        // Test get_graph_metrics
        let metrics = handler.get_graph_metrics(graph_id).await.unwrap();
        assert_eq!(metrics.node_count, 1);
        assert_eq!(metrics.edge_count, 0);
        assert_eq!(metrics.density, 0.0); // No edges in a single node graph
        assert_eq!(metrics.average_degree, 0.0); // No edges
    }

    #[tokio::test]
    async fn test_filter_graphs() {
        use crate::domain_events::GraphDomainEvent;
        use crate::events::GraphCreated;
        use crate::projections::{GraphProjection, GraphSummaryProjection, NodeListProjection};
        use chrono::{Duration, Utc};

        let mut graph_summary = GraphSummaryProjection::new();
        let node_list = NodeListProjection::new();

        let now = Utc::now();
        let past = now - Duration::days(1);
        let _future = now + Duration::days(1);

        // Add two test graphs with different creation times
        let graph1_id = GraphId::new();
        let graph2_id = GraphId::new();

        let graph1_event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id: graph1_id,
            name: "Early Graph".to_string(),
            description: "Created in the past".to_string(),
            graph_type: None,
            metadata: std::collections::HashMap::new(),
            created_at: past,
        });

        let graph2_event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id: graph2_id,
            name: "Recent Graph".to_string(),
            description: "Created more recently".to_string(),
            graph_type: None,
            metadata: std::collections::HashMap::new(),
            created_at: now,
        });

        graph_summary
            .handle_graph_event(graph1_event)
            .await
            .unwrap();
        graph_summary
            .handle_graph_event(graph2_event)
            .await
            .unwrap();

        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, crate::projections::EdgeListProjection::new());

        // Test date filtering
        let filter = FilterParams {
            created_after: Some(past + Duration::hours(12)),
            created_before: None,
            name_contains: None,
            node_types: None,
            edge_types: None,
        };

        let filtered = handler
            .filter_graphs(filter, PaginationParams::default())
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Recent Graph");

        // Test name filtering
        let name_filter = FilterParams {
            created_after: None,
            created_before: None,
            name_contains: Some("early".to_string()),
            node_types: None,
            edge_types: None,
        };

        let name_filtered = handler
            .filter_graphs(name_filter, PaginationParams::default())
            .await
            .unwrap();
        assert_eq!(name_filtered.len(), 1);
        assert_eq!(name_filtered[0].name, "Early Graph");
    }

    #[tokio::test]
    async fn test_pagination() {
        use crate::domain_events::GraphDomainEvent;
        use crate::events::GraphCreated;
        use crate::projections::{GraphProjection, GraphSummaryProjection, NodeListProjection};
        use chrono::Utc;

        let mut graph_summary = GraphSummaryProjection::new();
        let node_list = NodeListProjection::new();

        // Add multiple graphs
        for i in 0..5 {
            let graph_id = GraphId::new();
            let event = GraphDomainEvent::GraphCreated(GraphCreated {
                graph_id,
                name: format!("Graph {i}"),
                description: format!("Test graph number {i}"),
                graph_type: None,
                metadata: std::collections::HashMap::new(),
                created_at: Utc::now(),
            });

            graph_summary.handle_graph_event(event).await.unwrap();
        }

        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, crate::projections::EdgeListProjection::new());

        // Test pagination - first page
        let first_page = handler
            .get_all_graphs(PaginationParams {
                offset: 0,
                limit: 3,
            })
            .await
            .unwrap();
        assert_eq!(first_page.len(), 3);

        // Test pagination - second page
        let second_page = handler
            .get_all_graphs(PaginationParams {
                offset: 3,
                limit: 3,
            })
            .await
            .unwrap();
        assert_eq!(second_page.len(), 2);

        // Test pagination beyond available data
        let empty_page = handler
            .get_all_graphs(PaginationParams {
                offset: 10,
                limit: 3,
            })
            .await
            .unwrap();
        assert_eq!(empty_page.len(), 0);
    }

    #[tokio::test]
    async fn test_error_cases() {
        let handler = GraphQueryHandlerImpl::new();

        let nonexistent_graph = GraphId::new();
        let nonexistent_node = NodeId::new();

        // Test graph not found
        let graph_result = handler.get_graph(nonexistent_graph).await;
        assert!(matches!(
            graph_result,
            Err(GraphQueryError::GraphNotFound(_))
        ));

        // Test node not found
        let node_result = handler.get_node(nonexistent_node).await;
        assert!(matches!(node_result, Err(GraphQueryError::NodeNotFound(_))));

        // Test metrics for nonexistent graph
        let metrics_result = handler.get_graph_metrics(nonexistent_graph).await;
        assert!(matches!(
            metrics_result,
            Err(GraphQueryError::GraphNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_edge_queries() {
        // Create test projections
        let mut graph_summary = crate::projections::GraphSummaryProjection::new();
        let mut node_list = crate::projections::NodeListProjection::new();
        let mut edge_list = crate::projections::EdgeListProjection::new();

        let graph_id = GraphId::new();
        let node1_id = NodeId::new();
        let node2_id = NodeId::new();
        let edge_id = EdgeId::new();

        // Create graph
        graph_summary
            .handle_graph_event(GraphDomainEvent::GraphCreated(GraphCreated {
                graph_id,
                name: "Test Graph".to_string(),
                description: "Test".to_string(),
                graph_type: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            }))
            .await
            .unwrap();

        // Add nodes
        node_list
            .handle_graph_event(GraphDomainEvent::NodeAdded(NodeAdded {
                graph_id,
                node_id: node1_id,
                position: Position3D::default(),
                node_type: "TestNode".to_string(),
                metadata: HashMap::new(),
            }))
            .await
            .unwrap();

        node_list
            .handle_graph_event(GraphDomainEvent::NodeAdded(NodeAdded {
                graph_id,
                node_id: node2_id,
                position: Position3D::default(),
                node_type: "TestNode".to_string(),
                metadata: HashMap::new(),
            }))
            .await
            .unwrap();

        // Add edge
        edge_list
            .handle_graph_event(GraphDomainEvent::EdgeAdded(EdgeAdded {
                graph_id,
                edge_id,
                source: node1_id,
                target: node2_id,
                relationship: EdgeRelationship::Dependency {
                    dependency_type: "test".to_string(),
                    strength: 1.0,
                },
                edge_type: "dependency".to_string(),
                metadata: HashMap::new(),
            }))
            .await
            .unwrap();

        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, edge_list);

        // Test get_edge
        let edge_info = handler.get_edge(edge_id).await.unwrap();
        assert_eq!(edge_info.edge_id, edge_id);
        assert_eq!(edge_info.source_id, node1_id);
        assert_eq!(edge_info.target_id, node2_id);
        assert_eq!(edge_info.edge_type, "dependency");

        // Test get_edges_in_graph
        let edges = handler.get_edges_in_graph(graph_id).await.unwrap();
        assert_eq!(edges.len(), 1);

        // Test get_edges_by_type
        let typed_edges = handler
            .get_edges_by_type(graph_id, "dependency")
            .await
            .unwrap();
        assert_eq!(typed_edges.len(), 1);

        // Test get_node_edges
        let node_edges = handler.get_node_edges(node1_id).await.unwrap();
        assert_eq!(node_edges.len(), 1);

        // Test get_outgoing_edges
        let outgoing = handler.get_outgoing_edges(node1_id).await.unwrap();
        assert_eq!(outgoing.len(), 1);

        // Test get_incoming_edges
        let incoming = handler.get_incoming_edges(node2_id).await.unwrap();
        assert_eq!(incoming.len(), 1);
    }

    #[tokio::test]
    async fn test_graph_algorithms() {
        // Create test projections
        let mut graph_summary = crate::projections::GraphSummaryProjection::new();
        let mut node_list = crate::projections::NodeListProjection::new();
        let mut edge_list = crate::projections::EdgeListProjection::new();

        let graph_id = GraphId::new();
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();
        let node4 = NodeId::new();

        // Create graph
        graph_summary
            .handle_graph_event(GraphDomainEvent::GraphCreated(GraphCreated {
                graph_id,
                name: "Test Graph".to_string(),
                description: "Test".to_string(),
                graph_type: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            }))
            .await
            .unwrap();

        // Add nodes
        for node_id in [node1, node2, node3, node4] {
            node_list
                .handle_graph_event(GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: Position3D::default(),
                    node_type: "TestNode".to_string(),
                    metadata: HashMap::new(),
                }))
                .await
                .unwrap();
        }

        // Create edges: 1->2, 2->3, 3->1 (cycle), 4 (isolated)
        let edges = vec![
            (node1, node2),
            (node2, node3),
            (node3, node1),
        ];

        for (source, target) in edges {
            edge_list
                .handle_graph_event(GraphDomainEvent::EdgeAdded(EdgeAdded {
                    graph_id,
                    edge_id: EdgeId::new(),
                    source,
                    target,
                    relationship: EdgeRelationship::Dependency {
                        dependency_type: "test".to_string(),
                        strength: 1.0,
                    },
                    edge_type: "dependency".to_string(),
                    metadata: HashMap::new(),
                }))
                .await
                .unwrap();
        }

        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, edge_list);

        // Test has_cycles - should be true
        let has_cycles = handler.has_cycles(graph_id).await.unwrap();
        assert!(has_cycles, "Graph should have cycles");

        // Test find_shortest_path
        let path = handler
            .find_shortest_path(graph_id, node1, node3)
            .await
            .unwrap();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3); // 1->2->3

        // Test find_connected_components - should have 2 components
        let components = handler
            .find_connected_components(graph_id)
            .await
            .unwrap();
        assert_eq!(components.len(), 2); // One with 3 nodes, one with node4
    }

    #[tokio::test]
    async fn test_spatial_queries() {
        // Create test projections
        let mut graph_summary = crate::projections::GraphSummaryProjection::new();
        let mut node_list = crate::projections::NodeListProjection::new();
        let edge_list = crate::projections::EdgeListProjection::new();

        let graph_id = GraphId::new();

        // Create graph
        graph_summary
            .handle_graph_event(GraphDomainEvent::GraphCreated(GraphCreated {
                graph_id,
                name: "Test Graph".to_string(),
                description: "Test".to_string(),
                graph_type: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            }))
            .await
            .unwrap();

        // Add nodes with positions
        let positions = vec![
            (0.0, 0.0),   // At origin
            (3.0, 4.0),   // Distance 5 from origin
            (10.0, 0.0),  // Distance 10 from origin
        ];

        for (i, (x, y)) in positions.iter().enumerate() {
            let mut metadata = HashMap::new();
            metadata.insert("position_x".to_string(), serde_json::json!(x));
            metadata.insert("position_y".to_string(), serde_json::json!(y));

            node_list
                .handle_graph_event(GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id: NodeId::new(),
                    position: Position3D::default(),
                    node_type: format!("Node{i}"),
                    metadata,
                }))
                .await
                .unwrap();
        }

        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, edge_list);

        // Test find_nodes_near_position
        let center = Position2D { x: 0.0, y: 0.0 };
        let nearby = handler
            .find_nodes_near_position(graph_id, center, 6.0)
            .await
            .unwrap();

        assert_eq!(nearby.len(), 2); // Should find nodes at (0,0) and (3,4)
    }

    #[tokio::test]
    async fn test_source_sink_nodes() {
        // Create test projections
        let mut graph_summary = crate::projections::GraphSummaryProjection::new();
        let mut node_list = crate::projections::NodeListProjection::new();
        let mut edge_list = crate::projections::EdgeListProjection::new();

        let graph_id = GraphId::new();
        let source_node = NodeId::new();
        let middle_node = NodeId::new();
        let sink_node = NodeId::new();
        let isolated_node = NodeId::new();

        // Create graph
        graph_summary
            .handle_graph_event(GraphDomainEvent::GraphCreated(GraphCreated {
                graph_id,
                name: "Test Graph".to_string(),
                description: "Test".to_string(),
                graph_type: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            }))
            .await
            .unwrap();

        // Add nodes: source -> middle -> sink, and isolated
        for (node_id, node_type) in [
            (source_node, "source"),
            (middle_node, "middle"),
            (sink_node, "sink"),
            (isolated_node, "isolated"),
        ] {
            node_list
                .handle_graph_event(GraphDomainEvent::NodeAdded(NodeAdded {
                    graph_id,
                    node_id,
                    position: Position3D::default(),
                    node_type: node_type.to_string(),
                    metadata: HashMap::new(),
                }))
                .await
                .unwrap();
        }

        // Create edges: source -> middle -> sink
        edge_list
            .handle_graph_event(GraphDomainEvent::EdgeAdded(EdgeAdded {
                graph_id,
                edge_id: EdgeId::new(),
                source: source_node,
                target: middle_node,
                relationship: EdgeRelationship::Dependency {
                    dependency_type: "test".to_string(),
                    strength: 1.0,
                },
                edge_type: "dependency".to_string(),
                metadata: HashMap::new(),
            }))
            .await
            .unwrap();

        edge_list
            .handle_graph_event(GraphDomainEvent::EdgeAdded(EdgeAdded {
                graph_id,
                edge_id: EdgeId::new(),
                source: middle_node,
                target: sink_node,
                relationship: EdgeRelationship::Dependency {
                    dependency_type: "test".to_string(),
                    strength: 1.0,
                },
                edge_type: "dependency".to_string(),
                metadata: HashMap::new(),
            }))
            .await
            .unwrap();

        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list, edge_list);

        // Test find_source_nodes - should find source_node and isolated_node
        let source_nodes = handler.find_source_nodes(graph_id).await.unwrap();
        assert_eq!(source_nodes.len(), 2);
        let source_ids: HashSet<NodeId> = source_nodes.iter().map(|n| n.node_id).collect();
        assert!(source_ids.contains(&source_node));
        assert!(source_ids.contains(&isolated_node));

        // Test find_sink_nodes - should find sink_node and isolated_node
        let sink_nodes = handler.find_sink_nodes(graph_id).await.unwrap();
        assert_eq!(sink_nodes.len(), 2);
        let sink_ids: HashSet<NodeId> = sink_nodes.iter().map(|n| n.node_id).collect();
        assert!(sink_ids.contains(&sink_node));
        assert!(sink_ids.contains(&isolated_node));
    }
}

// Export the abstract query handler module
mod abstract_query_handler;
pub use abstract_query_handler::{
    AbstractGraphQueryHandler, AbstractGraphQueryRepository, GraphStatistics,
};
