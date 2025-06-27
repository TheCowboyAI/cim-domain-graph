//! Bevy Plugin for Graph Domain
//!
//! This plugin integrates the graph domain with Bevy ECS.

use bevy_ecs::prelude::*;
use bevy_app::{App, Plugin, Update};
use std::sync::Arc;

use crate::{
    bridge::{GraphBridge, BridgeEvent},
    events::*,
    systems::*,
};

/// Resource that holds the graph bridge
#[derive(Resource)]
pub struct GraphBridgeResource {
    pub bridge: Arc<GraphBridge>,
}

/// Plugin that adds graph domain functionality to Bevy
pub struct GraphDomainPlugin {
    runtime_handle: tokio::runtime::Handle,
}

impl GraphDomainPlugin {
    /// Create a new graph domain plugin
    pub fn new(runtime_handle: tokio::runtime::Handle) -> Self {
        Self { runtime_handle }
    }
}

impl Plugin for GraphDomainPlugin {
    fn build(&self, app: &mut App) {
        // Create and insert the bridge
        let bridge = Arc::new(GraphBridge::new(self.runtime_handle.clone()));
        app.insert_resource(GraphBridgeResource { bridge });
        
        // Register events
        app.add_event::<GraphCreated>()
            .add_event::<GraphUpdated>()
            .add_event::<GraphArchived>()
            .add_event::<NodeAdded>()
            .add_event::<NodeUpdated>()
            .add_event::<NodeRemoved>()
            .add_event::<EdgeAdded>()
            .add_event::<EdgeUpdated>()
            .add_event::<EdgeRemoved>();
        
        // Add systems
        app.add_systems(
            Update,
            (
                // Bridge polling system
                poll_graph_events,
                // Graph lifecycle systems
                create_graph_system,
                update_graph_system,
                archive_graph_system,
                // Node management systems
                add_node_system,
                update_node_system,
                remove_node_system,
                // Edge management systems
                connect_nodes_system,
                update_edge_system,
                disconnect_nodes_system,
            )
                .chain()
                .in_set(GraphSystemSet),
        );
        
        // Add system sets
        app.configure_sets(Update, GraphSystemSet);
    }
}

/// System set for graph systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphSystemSet;

/// System that polls the bridge for events and forwards them to ECS
fn poll_graph_events(
    bridge: Res<GraphBridgeResource>,
    mut graph_created: EventWriter<GraphCreated>,
    mut graph_updated: EventWriter<GraphUpdated>,
    mut graph_archived: EventWriter<GraphArchived>,
    mut node_added: EventWriter<NodeAdded>,
    mut node_updated: EventWriter<NodeUpdated>,
    mut node_removed: EventWriter<NodeRemoved>,
    mut edge_added: EventWriter<EdgeAdded>,
    mut edge_updated: EventWriter<EdgeUpdated>,
    mut edge_removed: EventWriter<EdgeRemoved>,
) {
    let events = bridge.bridge.receive_events();
    
    for event in events {
        match event {
            BridgeEvent::GraphCreated(e) => { graph_created.write(e); },
            BridgeEvent::GraphUpdated(e) => { graph_updated.write(e); },
            BridgeEvent::GraphArchived(e) => { graph_archived.write(e); },
            BridgeEvent::NodeAdded(e) => { node_added.write(e); },
            BridgeEvent::NodeUpdated(e) => { node_updated.write(e); },
            BridgeEvent::NodeRemoved(e) => { node_removed.write(e); },
            BridgeEvent::EdgeAdded(e) => { edge_added.write(e); },
            BridgeEvent::EdgeUpdated(e) => { edge_updated.write(e); },
            BridgeEvent::EdgeRemoved(e) => { edge_removed.write(e); },
        }
    }
}

/// Helper trait to add graph domain to Bevy apps
pub trait GraphDomainExt {
    /// Add the graph domain plugin
    fn add_graph_domain(&mut self, runtime_handle: tokio::runtime::Handle) -> &mut Self;
}

impl GraphDomainExt for App {
    fn add_graph_domain(&mut self, runtime_handle: tokio::runtime::Handle) -> &mut Self {
        self.add_plugins(GraphDomainPlugin::new(runtime_handle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_creation() {
        let runtime = tokio::runtime::Handle::current();
        let plugin = GraphDomainPlugin::new(runtime);
        
        // Plugin should be created successfully
        let mut app = App::new();
        app.add_plugins(plugin);
        
        // Verify resources were added
        assert!(app.world().contains_resource::<GraphBridgeResource>());
    }
} 