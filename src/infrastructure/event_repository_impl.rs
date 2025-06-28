//! Abstract graph event repository implementation
//!
//! This provides a concrete implementation of the AbstractGraphEventRepository trait
//! that works with the unified repository for event-driven persistence.

use crate::{
    aggregate::abstract_graph::AbstractGraph, handlers::AbstractGraphEventRepository,
    handlers::UnifiedGraphRepository, GraphId,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Production implementation of AbstractGraphEventRepository
pub struct AbstractGraphEventRepositoryImpl {
    unified_repository: Arc<dyn UnifiedGraphRepository>,
}

impl AbstractGraphEventRepositoryImpl {
    /// Create a new event repository instance
    pub fn new(unified_repository: Arc<dyn UnifiedGraphRepository>) -> Self {
        Self { unified_repository }
    }
}

#[async_trait]
impl AbstractGraphEventRepository for AbstractGraphEventRepositoryImpl {
    async fn load_graph(&self, graph_id: GraphId) -> Result<Option<AbstractGraph>, String> {
        match self.unified_repository.load_graph(graph_id, None).await {
            Ok(graph) => Ok(Some(graph)),
            Err(crate::commands::GraphCommandError::GraphNotFound(_)) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn save_graph(&self, graph: &AbstractGraph) -> Result<(), String> {
        self.unified_repository
            .save_graph(graph)
            .await
            .map_err(|e| e.to_string())
    }

    async fn delete_graph(&self, graph_id: GraphId) -> Result<(), String> {
        // In event sourcing, we don't actually delete graphs
        // We would emit a GraphDeleted event instead
        tracing::warn!(
            "Delete graph requested for {} - this should emit a deletion event",
            graph_id
        );
        Ok(())
    }

    async fn get_graph_type(&self, graph_id: GraphId) -> Result<Option<String>, String> {
        self.unified_repository
            .get_graph_type(graph_id)
            .await
            .map_err(|e| e.to_string())
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

            // Get existing graph type or default to "context"
            let graph_type = graphs
                .get(&graph.id())
                .map(|(_, t)| t.clone())
                .unwrap_or_else(|| "context".to_string());

            graphs.insert(graph.id(), (graph.clone(), graph_type));
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
    async fn test_event_repository() {
        let unified_repo = Arc::new(MockUnifiedRepository::new());
        let event_repo = AbstractGraphEventRepositoryImpl::new(unified_repo.clone());

        let graph_id = GraphId::new();
        let graph = AbstractGraph::new(GraphType::new_workflow(graph_id, "Test Workflow"));

        // Test save graph
        event_repo.save_graph(&graph).await.unwrap();

        // Test load graph
        let loaded = event_repo.load_graph(graph_id).await.unwrap();
        assert!(loaded.is_some());
        let loaded_graph = loaded.unwrap();
        assert_eq!(loaded_graph.id(), graph_id);
        assert_eq!(loaded_graph.name(), "Test Workflow");

        // Test get graph type
        let graph_type = event_repo.get_graph_type(graph_id).await.unwrap();
        assert_eq!(graph_type, Some("context".to_string())); // Mock defaults to context

        // Test delete (should just log)
        event_repo.delete_graph(graph_id).await.unwrap();

        // Graph should still exist (delete is a no-op in event sourcing)
        let still_exists = event_repo.load_graph(graph_id).await.unwrap();
        assert!(still_exists.is_some());
    }
}
