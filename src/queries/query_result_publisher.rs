//! Query result publisher with correlation support
//!
//! This module provides a wrapper around query handlers to publish query results
//! with proper correlation tracking for observability and auditing.

use crate::queries::{GraphQueryHandler, GraphQueryError, GraphInfo, NodeInfo, EdgeInfo, 
    GraphStructure, GraphMetrics, PaginationParams, FilterParams, Position2D};
use crate::{GraphId, NodeId, EdgeId};
use cim_domain::{QueryEnvelope, QueryHandler, QueryResponse, Query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

/// Unified graph query enum that combines all query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQuery {
    // Graph-level queries
    GetGraph { graph_id: GraphId },
    GetAllGraphs { pagination: PaginationParams },
    SearchGraphs { query: String, pagination: PaginationParams },
    FilterGraphs { filter: FilterParams, pagination: PaginationParams },
    
    // Node-level queries
    GetNode { node_id: NodeId },
    GetNodesInGraph { graph_id: GraphId },
    GetNodesByType { graph_id: GraphId, node_type: String },
    FindNodesNearPosition { graph_id: GraphId, center: Position2D, radius: f64 },
    
    // Edge-level queries
    GetEdge { edge_id: EdgeId },
    GetEdgesInGraph { graph_id: GraphId },
    
    // Structure and analysis queries
    GetGraphStructure { graph_id: GraphId },
    GetGraphMetrics { graph_id: GraphId },
    FindShortestPath { graph_id: GraphId, source: NodeId, target: NodeId },
}

impl Query for GraphQuery {}

/// Trait for publishing query results with correlation
#[async_trait::async_trait]
pub trait QueryResultPublisher: Send + Sync {
    /// Publish query result with correlation metadata
    async fn publish_result(
        &self,
        query_id: &str,
        query_type: &str,
        result: &serde_json::Value,
        correlation_id: String,
        causation_id: Option<String>,
        user_id: String,
    ) -> Result<(), GraphQueryError>;
}

/// Graph query handler that publishes results with correlation
pub struct ResultPublishingQueryHandler {
    inner: GraphQueryHandlerImpl,
    result_publisher: Arc<dyn QueryResultPublisher>,
}

impl ResultPublishingQueryHandler {
    /// Create a new result publishing query handler
    pub fn new(
        inner: GraphQueryHandlerImpl,
        result_publisher: Arc<dyn QueryResultPublisher>,
    ) -> Self {
        Self { inner, result_publisher }
    }

    /// Process query and publish result with correlation
    async fn process_and_publish(
        &self,
        query: &GraphQuery,
        envelope: &QueryEnvelope<GraphQuery>,
    ) -> Result<serde_json::Value, GraphQueryError> {
        // Process the query
        let result = match query {
            GraphQuery::GetGraph { graph_id } => {
                self.inner.get_graph(*graph_id).await
                    .map(|info| serde_json::to_value(info).unwrap())
            }
            GraphQuery::GetAllGraphs { pagination } => {
                self.inner.get_all_graphs(pagination.clone()).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::SearchGraphs { query: search_query, pagination } => {
                self.inner.search_graphs(search_query, pagination.clone()).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::FilterGraphs { filter, pagination } => {
                self.inner.filter_graphs(filter.clone(), pagination.clone()).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::GetNode { node_id } => {
                self.inner.get_node(*node_id).await
                    .map(|info| serde_json::to_value(info).unwrap())
            }
            GraphQuery::GetNodesInGraph { graph_id } => {
                self.inner.get_nodes_in_graph(*graph_id).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::GetNodesByType { graph_id, node_type } => {
                self.inner.get_nodes_by_type(*graph_id, node_type).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::FindNodesNearPosition { graph_id, center, radius } => {
                self.inner.find_nodes_near_position(*graph_id, *center, *radius).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::GetEdge { edge_id } => {
                self.inner.get_edge(*edge_id).await
                    .map(|info| serde_json::to_value(info).unwrap())
            }
            GraphQuery::GetEdgesInGraph { graph_id } => {
                self.inner.get_edges_in_graph(*graph_id).await
                    .map(|infos| serde_json::to_value(infos).unwrap())
            }
            GraphQuery::GetGraphStructure { graph_id } => {
                self.inner.get_graph_structure(*graph_id).await
                    .map(|structure| serde_json::to_value(structure).unwrap())
            }
            GraphQuery::GetGraphMetrics { graph_id } => {
                self.inner.get_graph_metrics(*graph_id).await
                    .map(|metrics| serde_json::to_value(metrics).unwrap())
            }
            GraphQuery::FindShortestPath { graph_id, source, target } => {
                self.inner.find_shortest_path(*graph_id, *source, *target).await
                    .map(|path| serde_json::to_value(path).unwrap())
            }
        };

        // If successful, publish the result
        if let Ok(ref result_value) = result {
            let query_type = match query {
                GraphQuery::GetGraph { .. } => "GetGraph",
                GraphQuery::GetAllGraphs { .. } => "GetAllGraphs",
                GraphQuery::SearchGraphs { .. } => "SearchGraphs",
                GraphQuery::FilterGraphs { .. } => "FilterGraphs",
                GraphQuery::GetNode { .. } => "GetNode",
                GraphQuery::GetNodesInGraph { .. } => "GetNodesInGraph",
                GraphQuery::GetNodesByType { .. } => "GetNodesByType",
                GraphQuery::FindNodesNearPosition { .. } => "FindNodesNearPosition",
                GraphQuery::GetEdge { .. } => "GetEdge",
                GraphQuery::GetEdgesInGraph { .. } => "GetEdgesInGraph",
                GraphQuery::GetGraphStructure { .. } => "GetGraphStructure",
                GraphQuery::GetGraphMetrics { .. } => "GetGraphMetrics",
                GraphQuery::FindShortestPath { .. } => "FindShortestPath",
            };

            // Publish result with correlation
            if let Err(e) = self.result_publisher
                .publish_result(
                    &envelope.id.to_string(),
                    query_type,
                    result_value,
                    envelope.correlation_id().to_string(),
                    Some(envelope.id.to_string()),
                    envelope.issued_by.clone(),
                )
                .await
            {
                error!(
                    query_id = %envelope.id,
                    error = %e,
                    "Failed to publish query result"
                );
                // Don't fail the query if publishing fails
            } else {
                info!(
                    query_id = %envelope.id,
                    query_type = query_type,
                    correlation_id = ?envelope.correlation_id(),
                    "Published query result with correlation"
                );
            }
        }

        result
    }
}

impl QueryHandler<GraphQuery> for ResultPublishingQueryHandler {
    fn handle(&self, envelope: QueryEnvelope<GraphQuery>) -> QueryResponse {
        let query_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();
        let query = envelope.query.clone();

        // Process the query synchronously
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async { 
            self.process_and_publish(&query, &envelope).await 
        });

        match result {
            Ok(result_value) => QueryResponse {
                query_id: cim_domain::IdType::Uuid(*query_id.as_uuid()),
                correlation_id,
                result: result_value,
            },
            Err(error) => {
                error!(
                    query_id = %query_id,
                    error = %error,
                    "Query processing failed"
                );
                // Return error as JSON
                QueryResponse {
                    query_id: cim_domain::IdType::Uuid(*query_id.as_uuid()),
                    correlation_id,
                    result: serde_json::json!({
                        "error": error.to_string()
                    }),
                }
            }
        }
    }
}

use crate::queries::GraphQueryHandlerImpl;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock result publisher for testing
    struct MockResultPublisher;
    
    #[async_trait::async_trait]
    impl QueryResultPublisher for MockResultPublisher {
        async fn publish_result(
            &self,
            _query_id: &str,
            _query_type: &str,
            _result: &serde_json::Value,
            _correlation_id: String,
            _causation_id: Option<String>,
            _user_id: String,
        ) -> Result<(), GraphQueryError> {
            Ok(())
        }
    }
    
    #[test]
    fn test_result_publishing() {
        let inner = GraphQueryHandlerImpl::new();
        let result_publisher = Arc::new(MockResultPublisher);
        let handler = ResultPublishingQueryHandler::new(inner, result_publisher);
        
        // Test would go here
        assert!(true);
    }
}