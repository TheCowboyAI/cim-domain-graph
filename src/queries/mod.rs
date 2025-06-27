//! Graph queries
//!
//! Queries provide read-only access to graph data. They operate on projections
//! and read models rather than directly on aggregates.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{GraphId, NodeId, EdgeId};
use crate::value_objects::{Position2D, Position3D};
use cim_domain::{Query, QueryEnvelope, QueryHandler, QueryResponse};

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
    SearchGraphs { query: String, pagination: PaginationParams },
    /// Filter graphs by criteria
    FilterGraphs { filter: FilterParams, pagination: PaginationParams },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeQuery {
    /// Get node information by ID
    GetNode { node_id: NodeId },
    /// Get all nodes in a graph
    GetNodesInGraph { graph_id: GraphId },
    /// Get nodes by type
    GetNodesByType { graph_id: GraphId, node_type: String },
    /// Find nodes within a radius of a position
    FindNodesNearPosition { graph_id: GraphId, center: Position2D, radius: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeQuery {
    /// Get edge information by ID
    GetEdge { edge_id: EdgeId },
    /// Get all edges in a graph
    GetEdgesInGraph { graph_id: GraphId },
    /// Get edges by type
    GetEdgesByType { graph_id: GraphId, edge_type: String },
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
    async fn get_all_graphs(&self, pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>>;
    
    /// Search graphs by name or description
    async fn search_graphs(&self, query: &str, pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>>;
    
    /// Filter graphs by criteria
    async fn filter_graphs(&self, filter: FilterParams, pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>>;
    
    // Node-level queries
    
    /// Get node information by ID
    async fn get_node(&self, node_id: NodeId) -> GraphQueryResult<NodeInfo>;
    
    /// Get all nodes in a graph
    async fn get_nodes_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>>;
    
    /// Get nodes by type
    async fn get_nodes_by_type(&self, graph_id: GraphId, node_type: &str) -> GraphQueryResult<Vec<NodeInfo>>;
    
    /// Find nodes within a radius of a position
    async fn find_nodes_near_position(
        &self, 
        graph_id: GraphId, 
        center: Position2D, 
        radius: f64
    ) -> GraphQueryResult<Vec<NodeInfo>>;
    
    // Edge-level queries
    
    /// Get edge information by ID
    async fn get_edge(&self, edge_id: EdgeId) -> GraphQueryResult<EdgeInfo>;
    
    /// Get all edges in a graph
    async fn get_edges_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<EdgeInfo>>;
    
    /// Get edges by type
    async fn get_edges_by_type(&self, graph_id: GraphId, edge_type: &str) -> GraphQueryResult<Vec<EdgeInfo>>;
    
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
    async fn find_connected_components(&self, graph_id: GraphId) -> GraphQueryResult<Vec<Vec<NodeId>>>;
    
    /// Find shortest path between two nodes
    async fn find_shortest_path(
        &self, 
        graph_id: GraphId, 
        source: NodeId, 
        target: NodeId
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
        }
    }

    /// Create with existing projections
    pub fn with_projections(
        graph_summary_projection: crate::projections::GraphSummaryProjection,
        node_list_projection: crate::projections::NodeListProjection,
    ) -> Self {
        Self {
            graph_summary_projection,
            node_list_projection,
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
                    self.search_graphs(query, pagination.clone()).await.map(|infos| {
                        // TODO: Publish result to event stream with correlation
                        serde_json::to_value(infos).unwrap()
                    })
                }
                GraphQuery::FilterGraphs { filter, pagination } => {
                    self.filter_graphs(filter.clone(), pagination.clone()).await.map(|infos| {
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
                NodeQuery::GetNodesByType { graph_id, node_type } => {
                    self.get_nodes_by_type(*graph_id, node_type).await.map(|infos| {
                        // TODO: Publish result to event stream with correlation
                        serde_json::to_value(infos).unwrap()
                    })
                }
                NodeQuery::FindNodesNearPosition { graph_id, center, radius } => {
                    self.find_nodes_near_position(*graph_id, *center, *radius).await.map(|infos| {
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
    
    async fn get_all_graphs(&self, pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>> {
        let summaries = self.graph_summary_projection
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
    
    async fn search_graphs(&self, query: &str, pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>> {
        let query_lower = query.to_lowercase();
        let all_summaries = self.graph_summary_projection.get_all_summaries();
        
        let matching_summaries: Vec<_> = all_summaries
            .into_iter()
            .filter(|summary| {
                summary.name.to_lowercase().contains(&query_lower) ||
                summary.description.to_lowercase().contains(&query_lower)
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
    
    async fn filter_graphs(&self, filter: FilterParams, pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>> {
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
                    if !summary.name.to_lowercase().contains(&name_filter.to_lowercase()) {
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
        let node_infos = self.node_list_projection
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
    
    async fn get_nodes_by_type(&self, graph_id: GraphId, node_type: &str) -> GraphQueryResult<Vec<NodeInfo>> {
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
        _graph_id: GraphId, 
        _center: Position2D, 
        _radius: f64
    ) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Implement spatial queries - requires position tracking in projections
        Ok(Vec::new())
    }
    
    async fn get_edge(&self, _edge_id: EdgeId) -> GraphQueryResult<EdgeInfo> {
        // TODO: Implement using edge projection when available
        Err(GraphQueryError::DataAccessError("Edge queries not yet implemented - missing edge projection".to_string()))
    }
    
    async fn get_edges_in_graph(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using edge list projection when available
        Ok(Vec::new())
    }
    
    async fn get_edges_by_type(&self, _graph_id: GraphId, _edge_type: &str) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement filtering by edge type - requires edge projection
        Ok(Vec::new())
    }
    
    async fn get_node_edges(&self, _node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using adjacency index - requires edge projection
        Ok(Vec::new())
    }
    
    async fn get_incoming_edges(&self, _node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using incoming edge index - requires edge projection
        Ok(Vec::new())
    }
    
    async fn get_outgoing_edges(&self, _node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using outgoing edge index - requires edge projection
        Ok(Vec::new())
    }
    
    async fn get_graph_structure(&self, graph_id: GraphId) -> GraphQueryResult<GraphStructure> {
        let nodes = self.get_nodes_in_graph(graph_id).await?;
        let edges = self.get_edges_in_graph(graph_id).await?;
        
        // Build adjacency list from edges
        let mut adjacency_list = HashMap::new();
        for edge in &edges {
            adjacency_list
                .entry(edge.source_id)
                .or_insert_with(Vec::new)
                .push(edge.target_id);
        }
        
        Ok(GraphStructure {
            nodes,
            edges,
            adjacency_list,
        })
    }
    
    async fn get_graph_metrics(&self, graph_id: GraphId) -> GraphQueryResult<GraphMetrics> {
        let summary = self.graph_summary_projection
            .get_summary(&graph_id)
            .ok_or(GraphQueryError::GraphNotFound(graph_id))?;
        
        // Calculate basic metrics from summary
        let node_count = summary.node_count;
        let edge_count = summary.edge_count;
        
        // Calculate density: edges / (nodes * (nodes - 1) / 2)
        let density = if node_count > 1 {
            (2.0 * edge_count as f64) / (node_count as f64 * (node_count as f64 - 1.0))
        } else {
            0.0
        };
        
        // Calculate average degree: 2 * edges / nodes
        let average_degree = if node_count > 0 {
            (2.0 * edge_count as f64) / node_count as f64
        } else {
            0.0
        };
        
        Ok(GraphMetrics {
            node_count,
            edge_count,
            density,
            average_degree,
            connected_components: 1, // TODO: Implement proper connected components analysis
            has_cycles: false,      // TODO: Implement cycle detection algorithm
        })
    }
    
    async fn find_connected_components(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<Vec<NodeId>>> {
        // TODO: Implement connected components algorithm using graph structure
        Ok(Vec::new())
    }
    
    async fn find_shortest_path(
        &self, 
        _graph_id: GraphId, 
        _source: NodeId, 
        _target: NodeId
    ) -> GraphQueryResult<Option<Vec<NodeId>>> {
        // TODO: Implement shortest path algorithm (e.g., Dijkstra or BFS)
        Ok(None)
    }
    
    async fn has_cycles(&self, _graph_id: GraphId) -> GraphQueryResult<bool> {
        // TODO: Implement cycle detection algorithm using DFS
        Ok(false)
    }
    
    async fn find_source_nodes(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Find nodes with no incoming edges - requires edge projection
        // For now, return all nodes as a placeholder
        let nodes = self.get_nodes_in_graph(graph_id).await?;
        Ok(nodes)
    }
    
    async fn find_sink_nodes(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Find nodes with no outgoing edges - requires edge projection  
        // For now, return all nodes as a placeholder
        let nodes = self.get_nodes_in_graph(graph_id).await?;
        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_query_handler_creation() {
        let handler = GraphQueryHandlerImpl::new();
        
        // Test that basic queries return appropriate responses
        let graphs = handler.get_all_graphs(PaginationParams::default()).await.unwrap();
        assert!(graphs.is_empty());
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
        
        let display = format!("{}", error);
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
        use crate::projections::{GraphSummaryProjection, NodeListProjection, GraphProjection};
        use crate::domain_events::GraphDomainEvent;
        use crate::events::{GraphCreated, NodeAdded};
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
            metadata: std::collections::HashMap::new(),
            created_at: now,
        });
        
        graph_summary.handle_graph_event(graph_event.clone()).await.unwrap();
        
        // Add test node
        let mut node_metadata = std::collections::HashMap::new();
        node_metadata.insert("name".to_string(), serde_json::Value::String("Test Node".to_string()));
        
        let node_event = GraphDomainEvent::NodeAdded(NodeAdded {
            graph_id,
            node_id,
            node_type: "TestType".to_string(),
            metadata: node_metadata,
        });
        
        // Handle node event in both projections
        graph_summary.handle_graph_event(node_event.clone()).await.unwrap();
        node_list.handle_graph_event(node_event).await.unwrap();
        
        // Create handler with test data
        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list);
        
        // Test get_graph
        let graph_info = handler.get_graph(graph_id).await.unwrap();
        assert_eq!(graph_info.name, "Test Graph");
        assert_eq!(graph_info.node_count, 1);
        assert_eq!(graph_info.edge_count, 0);
        
        // Test get_all_graphs
        let all_graphs = handler.get_all_graphs(PaginationParams::default()).await.unwrap();
        assert_eq!(all_graphs.len(), 1);
        assert_eq!(all_graphs[0].graph_id, graph_id);
        
        // Test search_graphs
        let search_results = handler.search_graphs("test", PaginationParams::default()).await.unwrap();
        assert_eq!(search_results.len(), 1);
        
        let no_results = handler.search_graphs("nonexistent", PaginationParams::default()).await.unwrap();
        assert_eq!(no_results.len(), 0);
        
        // Test get_node
        let node_info = handler.get_node(node_id).await.unwrap();
        assert_eq!(node_info.node_id, node_id);
        assert_eq!(node_info.graph_id, graph_id);
        assert_eq!(node_info.node_type, "TestType");
        
        // Test get_nodes_in_graph
        let nodes_in_graph = handler.get_nodes_in_graph(graph_id).await.unwrap();
        assert_eq!(nodes_in_graph.len(), 1);
        assert_eq!(nodes_in_graph[0].node_id, node_id);
        
        // Test get_nodes_by_type
        let typed_nodes = handler.get_nodes_by_type(graph_id, "TestType").await.unwrap();
        assert_eq!(typed_nodes.len(), 1);
        
        let no_typed_nodes = handler.get_nodes_by_type(graph_id, "NonExistentType").await.unwrap();
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
        use crate::projections::{GraphSummaryProjection, NodeListProjection, GraphProjection};
        use crate::domain_events::GraphDomainEvent;
        use crate::events::GraphCreated;
        use chrono::{Utc, Duration};

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
            metadata: std::collections::HashMap::new(),
            created_at: past,
        });
        
        let graph2_event = GraphDomainEvent::GraphCreated(GraphCreated {
            graph_id: graph2_id,
            name: "Recent Graph".to_string(),
            description: "Created more recently".to_string(),
            metadata: std::collections::HashMap::new(),
            created_at: now,
        });
        
        graph_summary.handle_graph_event(graph1_event).await.unwrap();
        graph_summary.handle_graph_event(graph2_event).await.unwrap();
        
        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list);
        
        // Test date filtering
        let filter = FilterParams {
            created_after: Some(past + Duration::hours(12)),
            created_before: None,
            name_contains: None,
            node_types: None,
            edge_types: None,
        };
        
        let filtered = handler.filter_graphs(filter, PaginationParams::default()).await.unwrap();
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
        
        let name_filtered = handler.filter_graphs(name_filter, PaginationParams::default()).await.unwrap();
        assert_eq!(name_filtered.len(), 1);
        assert_eq!(name_filtered[0].name, "Early Graph");
    }

    #[tokio::test]
    async fn test_pagination() {
        use crate::projections::{GraphSummaryProjection, NodeListProjection, GraphProjection};
        use crate::domain_events::GraphDomainEvent;
        use crate::events::GraphCreated;
        use chrono::Utc;

        let mut graph_summary = GraphSummaryProjection::new();
        let node_list = NodeListProjection::new();
        
        // Add multiple graphs
        for i in 0..5 {
            let graph_id = GraphId::new();
            let event = GraphDomainEvent::GraphCreated(GraphCreated {
                graph_id,
                name: format!("Graph {}", i),
                description: format!("Test graph number {}", i),
                metadata: std::collections::HashMap::new(),
                created_at: Utc::now(),
            });
            
            graph_summary.handle_graph_event(event).await.unwrap();
        }
        
        let handler = GraphQueryHandlerImpl::with_projections(graph_summary, node_list);
        
        // Test pagination - first page
        let first_page = handler.get_all_graphs(PaginationParams {
            offset: 0,
            limit: 3,
        }).await.unwrap();
        assert_eq!(first_page.len(), 3);
        
        // Test pagination - second page
        let second_page = handler.get_all_graphs(PaginationParams {
            offset: 3,
            limit: 3,
        }).await.unwrap();
        assert_eq!(second_page.len(), 2);
        
        // Test pagination beyond available data
        let empty_page = handler.get_all_graphs(PaginationParams {
            offset: 10,
            limit: 3,
        }).await.unwrap();
        assert_eq!(empty_page.len(), 0);
    }

    #[tokio::test]
    async fn test_error_cases() {
        let handler = GraphQueryHandlerImpl::new();
        
        let nonexistent_graph = GraphId::new();
        let nonexistent_node = NodeId::new();
        
        // Test graph not found
        let graph_result = handler.get_graph(nonexistent_graph).await;
        assert!(matches!(graph_result, Err(GraphQueryError::GraphNotFound(_))));
        
        // Test node not found
        let node_result = handler.get_node(nonexistent_node).await;
        assert!(matches!(node_result, Err(GraphQueryError::NodeNotFound(_))));
        
        // Test metrics for nonexistent graph
        let metrics_result = handler.get_graph_metrics(nonexistent_graph).await;
        assert!(matches!(metrics_result, Err(GraphQueryError::GraphNotFound(_))));
    }
}
