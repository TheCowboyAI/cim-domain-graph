//! Graph queries
//!
//! Queries provide read-only access to graph data. They operate on projections
//! and read models rather than directly on aggregates.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{GraphId, NodeId, EdgeId};
use crate::value_objects::{Position2D, Position3D};

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
            GraphQueryError::GraphNotFound(id) => write!(f, "Graph not found: {}", id),
            GraphQueryError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            GraphQueryError::EdgeNotFound(id) => write!(f, "Edge not found: {}", id),
            GraphQueryError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            GraphQueryError::DataAccessError(msg) => write!(f, "Data access error: {}", msg),
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

/// Implementation of graph query handler
pub struct GraphQueryHandlerImpl {
    // This would typically contain references to read models/projections
    // For now, we'll provide a basic implementation
}

impl GraphQueryHandlerImpl {
    /// Create a new graph query handler
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl GraphQueryHandler for GraphQueryHandlerImpl {
    async fn get_graph(&self, _graph_id: GraphId) -> GraphQueryResult<GraphInfo> {
        // TODO: Implement using graph summary projection
        Err(GraphQueryError::DataAccessError("Not implemented yet".to_string()))
    }
    
    async fn get_all_graphs(&self, _pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>> {
        // TODO: Implement using graph summary projection
        Ok(Vec::new())
    }
    
    async fn search_graphs(&self, _query: &str, _pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>> {
        // TODO: Implement search functionality
        Ok(Vec::new())
    }
    
    async fn filter_graphs(&self, _filter: FilterParams, _pagination: PaginationParams) -> GraphQueryResult<Vec<GraphInfo>> {
        // TODO: Implement filtering functionality
        Ok(Vec::new())
    }
    
    async fn get_node(&self, _node_id: NodeId) -> GraphQueryResult<NodeInfo> {
        // TODO: Implement using node projection
        Err(GraphQueryError::DataAccessError("Not implemented yet".to_string()))
    }
    
    async fn get_nodes_in_graph(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Implement using node list projection
        Ok(Vec::new())
    }
    
    async fn get_nodes_by_type(&self, _graph_id: GraphId, _node_type: &str) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Implement filtering by node type
        Ok(Vec::new())
    }
    
    async fn find_nodes_near_position(
        &self, 
        _graph_id: GraphId, 
        _center: Position2D, 
        _radius: f64
    ) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Implement spatial queries
        Ok(Vec::new())
    }
    
    async fn get_edge(&self, _edge_id: EdgeId) -> GraphQueryResult<EdgeInfo> {
        // TODO: Implement using edge projection
        Err(GraphQueryError::DataAccessError("Not implemented yet".to_string()))
    }
    
    async fn get_edges_in_graph(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using edge list projection
        Ok(Vec::new())
    }
    
    async fn get_edges_by_type(&self, _graph_id: GraphId, _edge_type: &str) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement filtering by edge type
        Ok(Vec::new())
    }
    
    async fn get_node_edges(&self, _node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using adjacency index
        Ok(Vec::new())
    }
    
    async fn get_incoming_edges(&self, _node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using incoming edge index
        Ok(Vec::new())
    }
    
    async fn get_outgoing_edges(&self, _node_id: NodeId) -> GraphQueryResult<Vec<EdgeInfo>> {
        // TODO: Implement using outgoing edge index
        Ok(Vec::new())
    }
    
    async fn get_graph_structure(&self, _graph_id: GraphId) -> GraphQueryResult<GraphStructure> {
        // TODO: Implement using combined projections
        Ok(GraphStructure {
            nodes: Vec::new(),
            edges: Vec::new(),
            adjacency_list: HashMap::new(),
        })
    }
    
    async fn get_graph_metrics(&self, _graph_id: GraphId) -> GraphQueryResult<GraphMetrics> {
        // TODO: Implement metrics calculation
        Ok(GraphMetrics {
            node_count: 0,
            edge_count: 0,
            density: 0.0,
            average_degree: 0.0,
            connected_components: 0,
            has_cycles: false,
        })
    }
    
    async fn find_connected_components(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<Vec<NodeId>>> {
        // TODO: Implement connected components algorithm
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
        // TODO: Implement cycle detection algorithm
        Ok(false)
    }
    
    async fn find_source_nodes(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Find nodes with no incoming edges
        Ok(Vec::new())
    }
    
    async fn find_sink_nodes(&self, _graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        // TODO: Find nodes with no outgoing edges
        Ok(Vec::new())
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
}
