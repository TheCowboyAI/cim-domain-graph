//! Example demonstrating integration of the graph abstraction layer with Bevy ECS

use bevy_app::{App, Startup, Update};
use bevy_ecs::prelude::*;
use cim_domain::{GraphId, NodeId, EdgeId};
use cim_domain_graph::{
    abstraction::{
        GraphType, GraphImplementation,
        DefaultGraphTransformer, GraphTransformer, TransformationOptions,
        DefaultGraphComposer, GraphComposer, CompositionOptions,
    },
    components::*,
    events::*,
};
use std::collections::HashMap;
use serde_json::json;

fn main() {
    println!("=== Graph Abstraction Integration Demo ===\n");
    
    // Create a Bevy app
    let mut app = App::new();
    
    // Add required events
    app.add_event::<GraphCreated>()
        .add_event::<GraphUpdated>()
        .add_event::<GraphArchived>()
        .add_event::<NodeAdded>()
        .add_event::<NodeRemoved>()
        .add_event::<EdgeAdded>()
        .add_event::<EdgeRemoved>();
    
    // Add demo systems
    app.add_systems(Startup, setup_demo);
    app.add_systems(Update, (
        demonstrate_transformation,
        demonstrate_composition,
    ).chain());
    
    // Run the app for a few updates
    println!("Running Bevy app with graph abstraction demonstration...\n");
    
    for i in 0..3 {
        println!("--- Update {} ---", i);
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("\nDemo complete!");
}

fn setup_demo(
    mut commands: Commands,
    mut graph_events: EventWriter<GraphCreated>,
    mut node_events: EventWriter<NodeAdded>,
    mut edge_events: EventWriter<EdgeAdded>,
) {
    println!("Setting up demo graphs...");
    
    // Create a workflow graph
    let workflow_id = GraphId::new();
    graph_events.write(GraphCreated {
        graph_id: workflow_id,
        name: "Order Processing Workflow".to_string(),
        description: "Main order processing workflow".to_string(),
        graph_type: Some(cim_domain_graph::components::GraphType::Workflow),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
    });
    
    // Add workflow nodes
    let receive_order = NodeId::new();
    let process_payment = NodeId::new();
    
    commands.spawn(NodeEntity {
        node_id: receive_order,
        graph_id: workflow_id,
    });
    
    node_events.write(NodeAdded {
        graph_id: workflow_id,
        node_id: receive_order,
        node_type: "Process".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Receive Order"));
            meta
        },
    });
    
    commands.spawn(NodeEntity {
        node_id: process_payment,
        graph_id: workflow_id,
    });
    
    node_events.write(NodeAdded {
        graph_id: workflow_id,
        node_id: process_payment,
        node_type: "Process".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Process Payment"));
            meta
        },
    });
    
    // Connect nodes
    let edge_id = EdgeId::new();
    commands.spawn(EdgeEntity {
        edge_id,
        source: receive_order,
        target: process_payment,
        graph_id: workflow_id,
    });
    
    edge_events.write(EdgeAdded {
        graph_id: workflow_id,
        edge_id,
        source: receive_order,
        target: process_payment,
        relationship: EdgeRelationship::Association {
            association_type: "sequence".to_string(),
        },
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    });
    
    // Create a knowledge graph
    let knowledge_id = GraphId::new();
    graph_events.write(GraphCreated {
        graph_id: knowledge_id,
        name: "E-commerce Concepts".to_string(),
        description: "Domain concepts for e-commerce".to_string(),
        graph_type: Some(cim_domain_graph::components::GraphType::Knowledge),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
    });
    
    // Store graph IDs for later use
    commands.insert_resource(DemoGraphs {
        _workflow_id: workflow_id,
        _knowledge_id: knowledge_id,
    });
    
    // Also store the graphs in a resource for demonstration
    let workflow_graph = GraphType::new_workflow(workflow_id, "Order Processing Workflow");
    let knowledge_graph = GraphType::new_concept(knowledge_id, "E-commerce Concepts");
    
    commands.insert_resource(DemoAbstractionGraphs {
        workflow: workflow_graph,
        knowledge: knowledge_graph,
    });
}

#[derive(Resource)]
struct DemoGraphs {
    _workflow_id: GraphId,
    _knowledge_id: GraphId,
}

#[derive(Resource)]
struct DemoAbstractionGraphs {
    workflow: GraphType,
    knowledge: GraphType,
}

fn demonstrate_transformation(
    graphs: Res<DemoAbstractionGraphs>,
    mut count: Local<u32>,
) {
    if *count > 0 {
        return;
    }
    *count += 1;
    
    println!("\nDemonstrating graph transformation...");
    
    // Transform workflow to concept graph
    let transformer = DefaultGraphTransformer::new();
    let options = TransformationOptions::default();
    
    match transformer.transform(&graphs.workflow, "concept", options) {
        Ok(concept_graph) => {
            println!("Successfully transformed workflow to concept graph!");
            println!("  - Nodes: {}", concept_graph.list_nodes().len());
            println!("  - Edges: {}", concept_graph.list_edges().len());
            
            // Show some node data
            for (node_id, node_data) in concept_graph.list_nodes().iter().take(2) {
                println!("  - Node {}: type={}, metadata={:?}", 
                    node_id, 
                    node_data.node_type,
                    node_data.metadata.get("name")
                );
            }
        }
        Err(e) => {
            println!("Transformation failed: {:?}", e);
        }
    }
}

fn demonstrate_composition(
    graphs: Res<DemoAbstractionGraphs>,
    mut count: Local<u32>,
) {
    if *count != 2 {
        *count += 1;
        return;
    }
    *count += 1;
    
    println!("\nDemonstrating graph composition...");
    
    // Compose workflow and knowledge graphs
    let composer = DefaultGraphComposer::new();
    let options = CompositionOptions::default();
    
    match composer.compose(&[&graphs.workflow, &graphs.knowledge], "context", options) {
        Ok(composed) => {
            println!("Successfully composed graphs!");
            println!("  - Total nodes: {}", composed.list_nodes().len());
            println!("  - Total edges: {}", composed.list_edges().len());
            println!("  - Graph type: Context (unified view)");
        }
        Err(e) => {
            println!("Composition failed: {:?}", e);
        }
    }
} 