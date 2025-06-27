//! Async-Sync Bridge for Graph Domain
//!
//! This module provides the bridge between async domain operations
//! and sync ECS systems in Bevy.

use crossbeam::channel::{bounded, Receiver, Sender};
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::{
    commands::GraphCommand,
    events::*,
    domain_events::GraphDomainEvent,
};

pub mod graph_bridge;

pub use graph_bridge::GraphBridge;

/// Capacity for command and event channels
const CHANNEL_CAPACITY: usize = 1000;

/// Bridge command that can be sent from sync to async
#[derive(Debug, Clone)]
pub enum BridgeCommand {
    GraphCommand(GraphCommand),
    Shutdown,
}

/// Bridge event that can be sent from async to sync
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    GraphCreated(GraphCreated),
    GraphUpdated(GraphUpdated),
    GraphArchived(GraphArchived),
    NodeAdded(NodeAdded),
    NodeUpdated(NodeUpdated),
    NodeRemoved(NodeRemoved),
    EdgeAdded(EdgeAdded),
    EdgeUpdated(EdgeUpdated),
    EdgeRemoved(EdgeRemoved),
}

impl From<GraphDomainEvent> for BridgeEvent {
    fn from(event: GraphDomainEvent) -> Self {
        match event {
            GraphDomainEvent::GraphCreated(e) => BridgeEvent::GraphCreated(e),
            GraphDomainEvent::NodeAdded(e) => BridgeEvent::NodeAdded(e),
            GraphDomainEvent::NodeRemoved(e) => BridgeEvent::NodeRemoved(e),
            GraphDomainEvent::EdgeAdded(e) => BridgeEvent::EdgeAdded(e),
            GraphDomainEvent::EdgeRemoved(e) => BridgeEvent::EdgeRemoved(e),
        }
    }
}

/// Async-Sync bridge for graph operations
pub struct AsyncSyncBridge {
    /// Commands: Bevy (sync) → Domain (async)
    command_tx: Sender<BridgeCommand>,
    command_rx: Arc<Mutex<Receiver<BridgeCommand>>>,
    
    /// Events: Domain (async) → Bevy (sync)
    event_tx: mpsc::UnboundedSender<BridgeEvent>,
    
    /// Sync receiver for Bevy
    sync_event_rx: Receiver<BridgeEvent>,
}

impl AsyncSyncBridge {
    /// Create a new async-sync bridge
    pub fn new() -> Self {
        // Command channel (sync → async)
        let (command_tx, command_rx) = bounded(CHANNEL_CAPACITY);
        
        // Event channel (async → sync)
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        
        // Sync event channel for Bevy
        let (sync_event_tx, sync_event_rx) = bounded(CHANNEL_CAPACITY);
        
        // Start forwarder task
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                if sync_event_tx.send(event).is_err() {
                    break; // Sync receiver dropped
                }
            }
        });
        
        Self {
            command_tx,
            command_rx: Arc::new(Mutex::new(command_rx)),
            event_tx,
            sync_event_rx,
        }
    }
    
    /// Send a command from sync to async
    pub fn send_command(&self, command: BridgeCommand) -> Result<(), SendError> {
        self.command_tx
            .send(command)
            .map_err(|_| SendError::ChannelClosed)
    }
    
    /// Receive commands in async context
    pub fn receive_command(&self) -> Option<BridgeCommand> {
        self.command_rx
            .lock()
            .try_recv()
            .ok()
    }
    
    /// Send an event from async to sync
    pub fn send_event(&self, event: BridgeEvent) -> Result<(), SendError> {
        self.event_tx
            .send(event)
            .map_err(|_| SendError::ChannelClosed)
    }
    
    /// Receive events in sync context (for Bevy)
    pub fn receive_events(&self) -> Vec<BridgeEvent> {
        let mut events = Vec::new();
        
        // Drain all available events
        while let Ok(event) = self.sync_event_rx.try_recv() {
            events.push(event);
            
            // Limit batch size to prevent frame drops
            if events.len() >= 100 {
                break;
            }
        }
        
        events
    }
    

}

impl Default for AsyncSyncBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for bridge operations
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Channel closed")]
    ChannelClosed,
} 