//! Example demonstrating integration of the graph abstraction layer with Bevy ECS

use bevy_app::{App, Startup, Update};
use bevy_ecs::prelude::*;
use cim_domain::{GraphId, NodeId, EdgeId};
use cim_domain_graph::{
    abstraction::{
        GraphType, GraphImplementation, NodeData, EdgeData,
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
    let ship_order = NodeId::new();
    
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
            meta.insert("duration".to_string(), json!("5 minutes"));
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
            meta.insert("duration".to_string(), json!("2 minutes"));
            meta
        },
    });
    
    commands.spawn(NodeEntity {
        node_id: ship_order,
        graph_id: workflow_id,
    });
    
    node_events.write(NodeAdded {
        graph_id: workflow_id,
        node_id: ship_order,
        node_type: "Process".to_string(),
        position: Position3D { x: 200.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Ship Order"));
            meta.insert("duration".to_string(), json!("1 day"));
            meta
        },
    });
    
    // Connect nodes
    let edge1_id = EdgeId::new();
    commands.spawn(EdgeEntity {
        edge_id: edge1_id,
        source: receive_order,
        target: process_payment,
        graph_id: workflow_id,
    });
    
    edge_events.write(EdgeAdded {
        graph_id: workflow_id,
        edge_id: edge1_id,
        source: receive_order,
        target: process_payment,
        relationship: EdgeRelationship::Association {
            association_type: "sequence".to_string(),
        },
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    });
    
    let edge2_id = EdgeId::new();
    commands.spawn(EdgeEntity {
        edge_id: edge2_id,
        source: process_payment,
        target: ship_order,
        graph_id: workflow_id,
    });
    
    edge_events.write(EdgeAdded {
        graph_id: workflow_id,
        edge_id: edge2_id,
        source: process_payment,
        target: ship_order,
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
    
    // Add knowledge nodes
    let order_concept = NodeId::new();
    let payment_concept = NodeId::new();
    let customer_concept = NodeId::new();
    
    commands.spawn(NodeEntity {
        node_id: order_concept,
        graph_id: knowledge_id,
    });
    
    node_events.write(NodeAdded {
        graph_id: knowledge_id,
        node_id: order_concept,
        node_type: "Concept".to_string(),
        position: Position3D { x: 0.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Order"));
            meta.insert("category".to_string(), json!("Transaction"));
            meta
        },
    });
    
    commands.spawn(NodeEntity {
        node_id: payment_concept,
        graph_id: knowledge_id,
    });
    
    node_events.write(NodeAdded {
        graph_id: knowledge_id,
        node_id: payment_concept,
        node_type: "Concept".to_string(),
        position: Position3D { x: 100.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Payment"));
            meta.insert("category".to_string(), json!("Finance"));
            meta
        },
    });
    
    commands.spawn(NodeEntity {
        node_id: customer_concept,
        graph_id: knowledge_id,
    });
    
    node_events.write(NodeAdded {
        graph_id: knowledge_id,
        node_id: customer_concept,
        node_type: "Concept".to_string(),
        position: Position3D { x: 50.0, y: 200.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Customer"));
            meta.insert("category".to_string(), json!("Entity"));
            meta
        },
    });
    
    // Connect knowledge nodes
    let k_edge1 = EdgeId::new();
    edge_events.write(EdgeAdded {
        graph_id: knowledge_id,
        edge_id: k_edge1,
        source: customer_concept,
        target: order_concept,
        relationship: EdgeRelationship::Association {
            association_type: "places".to_string(),
        },
        edge_type: "relationship".to_string(),
        metadata: HashMap::new(),
    });
    
    let k_edge2 = EdgeId::new();
    edge_events.write(EdgeAdded {
        graph_id: knowledge_id,
        edge_id: k_edge2,
        source: order_concept,
        target: payment_concept,
        relationship: EdgeRelationship::Association {
            association_type: "requires".to_string(),
        },
        edge_type: "relationship".to_string(),
        metadata: HashMap::new(),
    });
    
    // Create and populate GraphType instances with the actual data
    let mut workflow_graph = GraphType::new_workflow(workflow_id, "Order Processing Workflow");
    let mut knowledge_graph = GraphType::new_concept(knowledge_id, "E-commerce Concepts");
    
    // Populate workflow graph
    workflow_graph.add_node(receive_order, NodeData {
        node_type: "Process".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Receive Order"));
            meta.insert("duration".to_string(), json!("5 minutes"));
            meta
        },
    }).unwrap();
    
    workflow_graph.add_node(process_payment, NodeData {
        node_type: "Process".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Process Payment"));
            meta.insert("duration".to_string(), json!("2 minutes"));
            meta
        },
    }).unwrap();
    
    workflow_graph.add_node(ship_order, NodeData {
        node_type: "Process".to_string(),
        position: Position3D { x: 200.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Ship Order"));
            meta.insert("duration".to_string(), json!("1 day"));
            meta
        },
    }).unwrap();
    
    workflow_graph.add_edge(edge1_id, receive_order, process_payment, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    }).unwrap();
    
    workflow_graph.add_edge(edge2_id, process_payment, ship_order, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    }).unwrap();
    
    // Populate knowledge graph
    knowledge_graph.add_node(order_concept, NodeData {
        node_type: "Concept".to_string(),
        position: Position3D { x: 0.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Order"));
            meta.insert("category".to_string(), json!("Transaction"));
            meta
        },
    }).unwrap();
    
    knowledge_graph.add_node(payment_concept, NodeData {
        node_type: "Concept".to_string(),
        position: Position3D { x: 100.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Payment"));
            meta.insert("category".to_string(), json!("Finance"));
            meta
        },
    }).unwrap();
    
    knowledge_graph.add_node(customer_concept, NodeData {
        node_type: "Concept".to_string(),
        position: Position3D { x: 50.0, y: 200.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Customer"));
            meta.insert("category".to_string(), json!("Entity"));
            meta
        },
    }).unwrap();
    
    knowledge_graph.add_edge(k_edge1, customer_concept, order_concept, EdgeData {
        edge_type: "relationship".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("type".to_string(), json!("places"));
            meta
        },
    }).unwrap();
    
    knowledge_graph.add_edge(k_edge2, order_concept, payment_concept, EdgeData {
        edge_type: "relationship".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("type".to_string(), json!("requires"));
            meta
        },
    }).unwrap();
    
    // Store populated graphs for later use
    commands.insert_resource(DemoAbstractionGraphs {
        workflow: workflow_graph,
        knowledge: knowledge_graph,
    });
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
            for (node_id, node_data) in concept_graph.list_nodes().iter().take(3) {
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
            
            // Show composition details
            println!("\nComposed graph contents:");
            for (node_id, node_data) in composed.list_nodes().iter().take(5) {
                println!("  - Node {}: type={}, name={:?}", 
                    node_id, 
                    node_data.node_type,
                    node_data.metadata.get("name")
                );
            }
        }
        Err(e) => {
            println!("Composition failed: {:?}", e);
        }
    }
} 