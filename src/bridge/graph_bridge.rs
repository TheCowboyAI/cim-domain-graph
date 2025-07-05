//! Graph-specific bridge implementation
//!
//! Provides the integration between graph domain handlers and ECS systems.

use std::sync::Arc;
use tokio::runtime::Handle;

use crate::{
    handlers::{GraphCommandHandler, GraphCommandHandlerImpl, InMemoryGraphRepository},
    commands::GraphCommand,
    bridge::{AsyncSyncBridge, BridgeCommand, BridgeEvent},
};

/// Graph bridge that connects domain handlers with ECS
pub struct GraphBridge {
    bridge: Arc<AsyncSyncBridge>,
    _handler: Arc<GraphCommandHandlerImpl>,
    _runtime_handle: Handle,
}

impl GraphBridge {
    /// Create a new graph bridge
    pub fn new(runtime_handle: Handle) -> Self {
        let bridge = Arc::new(AsyncSyncBridge::new());
        let repository = Arc::new(InMemoryGraphRepository::new());
        let handler = Arc::new(GraphCommandHandlerImpl::new(repository));
        
        // Start command processor
        let bridge_clone = bridge.clone();
        let handler_clone = handler.clone();
        
        runtime_handle.spawn(async move {
            loop {
                if let Some(command) = bridge_clone.receive_command() {
                    match command {
                        BridgeCommand::GraphCommand(graph_cmd) => {
                            // Process command
                            match handler_clone.handle_graph_command(graph_cmd).await {
                                Ok(events) => {
                                    // Forward events to ECS
                                    for event in events {
                                        let bridge_event = BridgeEvent::from(event);
                                        if bridge_clone.send_event(bridge_event).is_err() {
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error handling graph command: {e:?}");
                                }
                            }
                        }
                        BridgeCommand::Shutdown => break,
                    }
                } else {
                    // No command available, yield
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                }
            }
        });
        
        Self {
            bridge,
            _handler: handler,
            _runtime_handle: runtime_handle,
        }
    }
    
    /// Get the async-sync bridge
    pub fn bridge(&self) -> &AsyncSyncBridge {
        &self.bridge
    }
    
    /// Send a graph command
    pub fn send_command(&self, command: GraphCommand) -> Result<(), crate::bridge::SendError> {
        self.bridge.send_command(BridgeCommand::GraphCommand(command))
    }
    
    /// Receive events for ECS processing
    pub fn receive_events(&self) -> Vec<BridgeEvent> {
        self.bridge.receive_events()
    }
    
    /// Shutdown the bridge
    pub fn shutdown(&self) -> Result<(), crate::bridge::SendError> {
        self.bridge.send_command(BridgeCommand::Shutdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_graph_bridge_command_processing() {
        let runtime = tokio::runtime::Handle::current();
        let bridge = GraphBridge::new(runtime);
        
        // Send a create graph command
        let command = GraphCommand::CreateGraph {
            name: "Test Graph".to_string(),
            description: "Test".to_string(),
            metadata: HashMap::new(),
        };
        
        bridge.send_command(command).unwrap();
        
        // Wait for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Receive events
        let events = bridge.receive_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            BridgeEvent::GraphCreated(e) => {
                assert_eq!(e.name, "Test Graph");
            }
            _ => panic!("Expected GraphCreated event"),
        }
    }
} 