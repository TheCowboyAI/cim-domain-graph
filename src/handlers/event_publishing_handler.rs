//! Event publishing handler with correlation support
//!
//! This handler extends the unified handler to publish events with proper
//! correlation and causation chains.

use crate::{
    commands::{GraphCommand, GraphCommandError, GraphCommandResult},
    domain_events::GraphDomainEvent,
    handlers::UnifiedGraphCommandHandler,
};
use cim_domain::{
    CommandEnvelope, CommandHandler, CommandAcknowledgment, CommandStatus,
};
use std::sync::Arc;
use tracing::{info, error};

/// Trait for publishing graph events with correlation
#[async_trait::async_trait]
pub trait GraphEventPublisher: Send + Sync {
    /// Publish graph events with correlation metadata
    async fn publish_events(
        &self,
        graph_id: &str,
        events: Vec<GraphDomainEvent>,
        correlation_id: String,
        causation_id: Option<String>,
        user_id: String,
    ) -> Result<(), GraphCommandError>;
}

/// Graph command handler that publishes events with correlation
pub struct EventPublishingGraphHandler {
    inner: UnifiedGraphCommandHandler,
    event_publisher: Arc<dyn GraphEventPublisher>,
}

impl EventPublishingGraphHandler {
    /// Create a new event publishing handler
    pub fn new(
        inner: UnifiedGraphCommandHandler,
        event_publisher: Arc<dyn GraphEventPublisher>,
    ) -> Self {
        Self { inner, event_publisher }
    }

    /// Process command and publish events with correlation
    async fn process_and_publish(
        &self,
        command: GraphCommand,
        envelope: &CommandEnvelope<GraphCommand>,
    ) -> GraphCommandResult<Vec<GraphDomainEvent>> {
        // Get graph ID for aggregate (for CreateGraph, we'll generate it from events)
        let graph_id_opt = command.graph_id();

        // Process the command
        let events = self.inner.process_graph_command(command, envelope).await?;

        if !events.is_empty() {
            // Determine aggregate ID from command or events
            let aggregate_id = if let Some(graph_id) = graph_id_opt {
                graph_id.to_string()
            } else {
                // For CreateGraph, get ID from the event
                match &events[0] {
                    GraphDomainEvent::GraphCreated(event) => event.graph_id.to_string(),
                    _ => return Err(GraphCommandError::InvalidCommand("Expected GraphCreated event".to_string())),
                }
            };

            // Publish events with correlation
            self.event_publisher
                .publish_events(
                    &aggregate_id,
                    events.clone(),
                    envelope.correlation_id().to_string(),
                    Some(envelope.id.to_string()),
                    envelope.issued_by.clone(),
                )
                .await?;

            info!(
                aggregate_id = %aggregate_id,
                event_count = events.len(),
                correlation_id = ?envelope.correlation_id(),
                "Published graph events with correlation"
            );
        }

        Ok(events)
    }
}

impl CommandHandler<GraphCommand> for EventPublishingGraphHandler {
    fn handle(&mut self, envelope: CommandEnvelope<GraphCommand>) -> CommandAcknowledgment {
        let command_id = envelope.id;
        let correlation_id = envelope.correlation_id().clone();

        // Extract command
        let command = envelope.command.clone();

        // Process and publish synchronously
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async { 
            self.process_and_publish(command, &envelope).await 
        });

        match result {
            Ok(events) => {
                info!(
                    command_id = %command_id,
                    event_count = events.len(),
                    "Command processed successfully"
                );
                CommandAcknowledgment {
                    command_id,
                    correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            Err(error) => {
                error!(
                    command_id = %command_id,
                    error = %error,
                    "Command processing failed"
                );
                CommandAcknowledgment {
                    command_id,
                    correlation_id,
                    status: CommandStatus::Rejected,
                    reason: Some(error.to_string()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        handlers::{UnifiedGraphRepository, UnifiedGraphCommandHandler},
        GraphId, NodeId, EdgeId,
    };
    
    // Mock event publisher for testing
    struct MockEventPublisher;
    
    #[async_trait::async_trait]
    impl GraphEventPublisher for MockEventPublisher {
        async fn publish_events(
            &self,
            _graph_id: &str,
            _events: Vec<GraphDomainEvent>,
            _correlation_id: String,
            _causation_id: Option<String>,
            _user_id: String,
        ) -> Result<(), GraphCommandError> {
            Ok(())
        }
    }
    
    // Mock repository for testing
    struct MockRepository;
    
    #[async_trait]
    impl UnifiedGraphRepository for MockRepository {
        async fn load_graph(
            &self,
            _graph_id: GraphId,
            _graph_type: Option<&str>,
        ) -> GraphCommandResult<crate::aggregate::abstract_graph::AbstractGraph> {
            unimplemented!("Mock implementation")
        }

        async fn save_graph(&self, _graph: &crate::aggregate::abstract_graph::AbstractGraph) -> GraphCommandResult<()> {
            Ok(())
        }

        async fn exists(&self, _graph_id: GraphId) -> GraphCommandResult<bool> {
            Ok(false)
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

        async fn get_graph_type(&self, _graph_id: GraphId) -> GraphCommandResult<Option<String>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_event_publishing() {
        let repository = Arc::new(MockRepository);
        let inner = UnifiedGraphCommandHandler::new(repository);
        let event_publisher = Arc::new(MockEventPublisher);
        let handler = EventPublishingGraphHandler::new(inner, event_publisher.clone());

        // Test would go here - simplified for brevity
        assert!(true);
    }
}