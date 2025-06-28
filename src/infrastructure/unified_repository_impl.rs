//! Unified graph repository implementation
//!
//! This provides a concrete implementation of the UnifiedGraphRepository trait
//! that works with NATS event store and projections.

use crate::{
    abstraction::GraphType,
    aggregate::AbstractGraph,
    commands::GraphCommandResult,
    handlers::UnifiedGraphRepository,
    projections::{GraphSummaryProjection, NodeListProjection},
    GraphId, NodeId, EdgeId,
};
use async_trait::async_trait;
use cim_domain::infrastructure::EventStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified repository implementation that combines all repository traits
pub struct UnifiedGraphRepositoryImpl {
    _event_store: Arc<dyn EventStore>,
    graph_summaries: Arc<RwLock<GraphSummaryProjection>>,
    _node_lists: Arc<RwLock<HashMap<GraphId, NodeListProjection>>>,
}

impl UnifiedGraphRepositoryImpl {
    /// Create a new unified repository
    pub fn new(
        event_store: Arc<dyn EventStore>,
        graph_summaries: Arc<RwLock<GraphSummaryProjection>>,
        node_lists: Arc<RwLock<HashMap<GraphId, NodeListProjection>>>,
    ) -> Self {
        Self {
            _event_store: event_store,
            graph_summaries,
            _node_lists: node_lists,
        }
    }

    /// Load events and rebuild aggregate
    async fn rebuild_aggregate(
        &self,
        graph_id: GraphId,
        graph_type: Option<&str>,
    ) -> GraphCommandResult<AbstractGraph> {
        // For now, we'll create a new graph with the appropriate type
        // In a real implementation, you'd rebuild from events
        let graph_type_enum = self.determine_graph_type(graph_id, graph_type).await?;
        let graph = AbstractGraph::new(graph_type_enum);

        // Log that we would rebuild from events
        tracing::debug!("Would rebuild graph {} from events", graph_id);

        Ok(graph)
    }

    /// Determine graph type from projections or hint
    async fn determine_graph_type(
        &self,
        graph_id: GraphId,
        graph_type_hint: Option<&str>,
    ) -> GraphCommandResult<GraphType> {
        // If we have a hint, use it
        if let Some(type_str) = graph_type_hint {
            let graph_type = match type_str {
                "context" => GraphType::new_context(graph_id, ""),
                "concept" => GraphType::new_concept(graph_id, ""),
                "workflow" => GraphType::new_workflow(graph_id, ""),
                "ipld" => GraphType::new_ipld(graph_id),
                _ => GraphType::new_context(graph_id, ""), // Default
            };
            return Ok(graph_type);
        }

        // Try to get from projection
        let summaries = self.graph_summaries.read().await;
        if let Some(summary) = summaries.get_graph_summary(&graph_id) {
            let graph_type = match summary.graph_type.as_deref() {
                Some("context") => GraphType::new_context(graph_id, &summary.name),
                Some("concept") => GraphType::new_concept(graph_id, &summary.name),
                Some("workflow") => GraphType::new_workflow(graph_id, &summary.name),
                Some("ipld") => GraphType::new_ipld(graph_id),
                _ => GraphType::new_context(graph_id, &summary.name), // Default
            };
            return Ok(graph_type);
        }

        // Default to context graph
        Ok(GraphType::new_context(graph_id, "Unknown"))
    }
}

#[async_trait]
impl UnifiedGraphRepository for UnifiedGraphRepositoryImpl {
    async fn load_graph(
        &self,
        graph_id: GraphId,
        graph_type: Option<&str>,
    ) -> GraphCommandResult<AbstractGraph> {
        self.rebuild_aggregate(graph_id, graph_type).await
    }

    async fn save_graph(&self, graph: &AbstractGraph) -> GraphCommandResult<()> {
        // In event sourcing, we don't save the aggregate directly
        // Events are already persisted when commands are processed
        tracing::debug!(
            "Graph {} state updated (events already persisted)",
            graph.id()
        );
        Ok(())
    }

    async fn exists(&self, graph_id: GraphId) -> GraphCommandResult<bool> {
        let summaries = self.graph_summaries.read().await;
        Ok(summaries.get_graph_summary(&graph_id).is_some())
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
        let summaries = self.graph_summaries.read().await;
        Ok(summaries
            .get_graph_summary(&graph_id)
            .and_then(|s| s.graph_type.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::GraphCreated;
    use crate::projections::GraphProjection;
    use cim_domain::infrastructure::event_store::EventStream;
    use std::collections::HashMap;

    // Mock event store for testing
    #[derive(Debug)]
    struct MockEventStore {
        events: RwLock<HashMap<String, Vec<cim_domain::infrastructure::StoredEvent>>>,
    }

    impl MockEventStore {
        fn new() -> Self {
            Self {
                events: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl EventStore for MockEventStore {
        async fn append_events(
            &self,
            aggregate_id: &str,
            aggregate_type: &str,
            events: Vec<cim_domain::DomainEventEnum>,
            _expected_version: Option<u64>,
            metadata: cim_domain::infrastructure::EventMetadata,
        ) -> Result<(), cim_domain::infrastructure::EventStoreError> {
            let mut store = self.events.write().await;
            let entry = store
                .entry(aggregate_id.to_string())
                .or_insert_with(Vec::new);

            for event in events {
                entry.push(cim_domain::infrastructure::StoredEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    aggregate_id: aggregate_id.to_string(),
                    aggregate_type: aggregate_type.to_string(),
                    sequence: entry.len() as u64 + 1,
                    event,
                    metadata: metadata.clone(),
                    stored_at: chrono::Utc::now(),
                });
            }

            Ok(())
        }

        async fn get_events(
            &self,
            aggregate_id: &str,
            after_sequence: Option<u64>,
        ) -> Result<
            Vec<cim_domain::infrastructure::StoredEvent>,
            cim_domain::infrastructure::EventStoreError,
        > {
            let store = self.events.read().await;
            Ok(store
                .get(aggregate_id)
                .map(|events| {
                    events
                        .iter()
                        .filter(|e| after_sequence.map_or(true, |seq| e.sequence > seq))
                        .cloned()
                        .collect()
                })
                .unwrap_or_default())
        }

        async fn get_events_by_type(
            &self,
            _event_type: &str,
            _limit: usize,
            _after: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<
            Vec<cim_domain::infrastructure::StoredEvent>,
            cim_domain::infrastructure::EventStoreError,
        > {
            unimplemented!("Not needed for tests")
        }

        async fn get_aggregate_version(
            &self,
            aggregate_id: &str,
        ) -> Result<Option<u64>, cim_domain::infrastructure::EventStoreError> {
            let store = self.events.read().await;
            Ok(store
                .get(aggregate_id)
                .and_then(|events| events.last())
                .map(|e| e.sequence))
        }

        async fn subscribe_to_events(
            &self,
            _from_sequence: Option<u64>,
        ) -> Result<Box<dyn EventStream>, cim_domain::infrastructure::EventStoreError> {
            unimplemented!("Not needed for tests")
        }

        async fn subscribe_to_aggregate_type(
            &self,
            _aggregate_type: &str,
            _from_sequence: Option<u64>,
        ) -> Result<Box<dyn EventStream>, cim_domain::infrastructure::EventStoreError> {
            unimplemented!("Not needed for tests")
        }

        async fn stream_events_by_type(
            &self,
            _event_type: &str,
            _from_sequence: Option<u64>,
        ) -> Result<Box<dyn EventStream>, cim_domain::infrastructure::EventStoreError> {
            unimplemented!("Not needed for tests")
        }

        async fn stream_all_events(
            &self,
            _from_sequence: Option<u64>,
        ) -> Result<Box<dyn EventStream>, cim_domain::infrastructure::EventStoreError> {
            unimplemented!("Not needed for tests")
        }
    }

    #[tokio::test]
    async fn test_repository_implementation() {
        let event_store = Arc::new(MockEventStore::new());
        let graph_summaries = Arc::new(RwLock::new(GraphSummaryProjection::new()));
        let node_lists = Arc::new(RwLock::new(HashMap::new()));

        let repository = UnifiedGraphRepositoryImpl::new(
            event_store.clone(),
            graph_summaries.clone(),
            node_lists.clone(),
        );

        let graph_id = GraphId::new();

        // Initially, graph doesn't exist
        assert!(!repository.exists(graph_id).await.unwrap());

        // Add graph to projection
        {
            let mut summaries = graph_summaries.write().await;
            summaries
                .handle_graph_event(crate::GraphDomainEvent::GraphCreated(GraphCreated {
                    graph_id,
                    name: "Test Graph".to_string(),
                    description: "".to_string(),
                    graph_type: Some(crate::components::GraphType::Workflow),
                    metadata: HashMap::new(),
                    created_at: chrono::Utc::now(),
                }))
                .await
                .unwrap();
        }

        // Now graph exists in projection
        assert!(repository.exists(graph_id).await.unwrap());
        assert_eq!(
            repository.get_graph_type(graph_id).await.unwrap(),
            Some("workflow".to_string())
        );
    }
}
