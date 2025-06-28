//! Abstract graph query repository implementation
//!
//! This provides a concrete implementation of the AbstractGraphQueryRepository trait
//! that works with the unified repository to load and query abstract graphs.

use crate::{
    aggregate::abstract_graph::AbstractGraph,
    handlers::UnifiedGraphRepository,
    queries::AbstractGraphQueryRepository,
    queries::{GraphQueryError, GraphQueryResult},
    GraphId,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Production implementation of AbstractGraphQueryRepository
pub struct AbstractGraphQueryRepositoryImpl {
    unified_repository: Arc<dyn UnifiedGraphRepository>,
}

impl AbstractGraphQueryRepositoryImpl {
    /// Create a new query repository instance
    pub fn new(unified_repository: Arc<dyn UnifiedGraphRepository>) -> Self {
        Self { unified_repository }
    }
}

#[async_trait]
impl AbstractGraphQueryRepository for AbstractGraphQueryRepositoryImpl {
    async fn load_abstract_graph(&self, graph_id: GraphId) -> GraphQueryResult<AbstractGraph> {
        self.unified_repository
            .load_graph(graph_id, None)
            .await
            .map_err(|e| GraphQueryError::DataAccessError(e.to_string()))
    }

    async fn get_graph_type(&self, graph_id: GraphId) -> GraphQueryResult<String> {
        self.unified_repository
            .get_graph_type(graph_id)
            .await
            .map_err(|e| GraphQueryError::DataAccessError(e.to_string()))?
            .ok_or(GraphQueryError::GraphNotFound(graph_id))
    }

    async fn list_graph_ids(&self) -> GraphQueryResult<Vec<GraphId>> {
        // This would need to be implemented with a proper projection
        // For now, return empty list
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        abstraction::GraphType,
        commands::{GraphCommandError, GraphCommandResult},
        EdgeId, NodeId,
    };
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    // Mock unified repository for testing
    struct MockUnifiedRepository {
        graphs: RwLock<HashMap<GraphId, (AbstractGraph, String)>>,
    }

    impl MockUnifiedRepository {
        fn new() -> Self {
            Self {
                graphs: RwLock::new(HashMap::new()),
            }
        }

        async fn add_graph(&self, graph: AbstractGraph, graph_type: String) {
            let mut graphs = self.graphs.write().await;
            graphs.insert(graph.id(), (graph, graph_type));
        }
    }

    #[async_trait]
    impl UnifiedGraphRepository for MockUnifiedRepository {
        async fn load_graph(
            &self,
            graph_id: GraphId,
            _graph_type: Option<&str>,
        ) -> GraphCommandResult<AbstractGraph> {
            let graphs = self.graphs.read().await;
            graphs
                .get(&graph_id)
                .map(|(g, _)| g.clone())
                .ok_or(GraphCommandError::GraphNotFound(graph_id))
        }

        async fn save_graph(&self, graph: &AbstractGraph) -> GraphCommandResult<()> {
            let mut graphs = self.graphs.write().await;
            if let Some((stored_graph, _)) = graphs.get_mut(&graph.id()) {
                *stored_graph = graph.clone();
            }
            Ok(())
        }

        async fn exists(&self, graph_id: GraphId) -> GraphCommandResult<bool> {
            let graphs = self.graphs.read().await;
            Ok(graphs.contains_key(&graph_id))
        }

        async fn next_graph_id(&self) -> GraphCommandResult<GraphId> {
            Ok(GraphId::new())
        }

        async fn next_node_id(&self) -> GraphCommandResult<NodeId> {
            Ok(NodeId::new())
        }

        async fn next_edge_id(&self) -> GraphCommandResult<EdgeId> {
            Ok(EdgeId::new())
        }

        async fn get_graph_type(&self, graph_id: GraphId) -> GraphCommandResult<Option<String>> {
            let graphs = self.graphs.read().await;
            Ok(graphs.get(&graph_id).map(|(_, t)| t.clone()))
        }
    }

    #[tokio::test]
    async fn test_query_repository() {
        let unified_repo = Arc::new(MockUnifiedRepository::new());
        let query_repo = AbstractGraphQueryRepositoryImpl::new(unified_repo.clone());

        let graph_id = GraphId::new();
        let graph = AbstractGraph::new(GraphType::new_context(graph_id, "Test Graph"));

        // Add graph to unified repository
        unified_repo
            .add_graph(graph.clone(), "context".to_string())
            .await;

        // Test load abstract graph
        let loaded = query_repo.load_abstract_graph(graph_id).await.unwrap();
        assert_eq!(loaded.id(), graph_id);
        assert_eq!(loaded.name(), "Test Graph");

        // Test get graph type
        let graph_type = query_repo.get_graph_type(graph_id).await.unwrap();
        assert_eq!(graph_type, "context");

        // Test non-existent graph
        let missing_id = GraphId::new();
        let result = query_repo.load_abstract_graph(missing_id).await;
        assert!(matches!(result, Err(GraphQueryError::DataAccessError(_))));
    }
}
