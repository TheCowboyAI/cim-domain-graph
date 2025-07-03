//! Example demonstrating graph composition functionality

use cim_domain::{NodeId, EdgeId, GraphId};
use cim_domain_graph::abstraction::{
    GraphType, GraphImplementation, NodeData, EdgeData, Position3D,
    DefaultGraphComposer, GraphComposer, CompositionOptions, ConflictResolution,
};
use std::collections::HashMap;
use serde_json::json;

fn main() {
    println!("=== Graph Composition Demo ===\n");
    
    // Create a graph composer
    let composer = DefaultGraphComposer::new();
    
    // Example 1: Basic composition of workflow graphs
    basic_composition_example(&composer);
    
    // Example 2: Handling conflicts with different strategies
    conflict_resolution_example(&composer);
    
    // Example 3: Cross-type composition
    cross_type_composition_example(&composer);
    
    // Example 4: Complex composition with custom mappings
    custom_mapping_example(&composer);
}

fn basic_composition_example(composer: &DefaultGraphComposer) {
    println!("1. Basic Composition Example");
    println!("----------------------------");
    
    // Create two workflow graphs representing different parts of a process
    let mut workflow1 = GraphType::new_workflow(GraphId::new(), "Order Processing");
    let mut workflow2 = GraphType::new_workflow(GraphId::new(), "Fulfillment");
    
    // Add nodes to workflow1
    let receive_order = NodeId::new();
    let validate_order = NodeId::new();
    let process_payment = NodeId::new();
    
    workflow1.add_node(receive_order, NodeData {
        node_type: "start".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Receive Order"));
            meta.insert("duration".to_string(), json!("5m"));
            meta
        },
    }).unwrap();
    
    workflow1.add_node(validate_order, NodeData {
        node_type: "process".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Validate Order"));
            meta.insert("duration".to_string(), json!("10m"));
            meta
        },
    }).unwrap();
    
    workflow1.add_node(process_payment, NodeData {
        node_type: "process".to_string(),
        position: Position3D { x: 200.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Process Payment"));
            meta.insert("duration".to_string(), json!("15m"));
            meta
        },
    }).unwrap();
    
    // Add edges to workflow1
    workflow1.add_edge(EdgeId::new(), receive_order, validate_order, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    }).unwrap();
    
    workflow1.add_edge(EdgeId::new(), validate_order, process_payment, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    }).unwrap();
    
    // Add nodes to workflow2
    let pick_items = NodeId::new();
    let pack_order = NodeId::new();
    let ship_order = NodeId::new();
    
    workflow2.add_node(pick_items, NodeData {
        node_type: "process".to_string(),
        position: Position3D { x: 300.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Pick Items"));
            meta.insert("duration".to_string(), json!("20m"));
            meta
        },
    }).unwrap();
    
    workflow2.add_node(pack_order, NodeData {
        node_type: "process".to_string(),
        position: Position3D { x: 400.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Pack Order"));
            meta.insert("duration".to_string(), json!("10m"));
            meta
        },
    }).unwrap();
    
    workflow2.add_node(ship_order, NodeData {
        node_type: "end".to_string(),
        position: Position3D { x: 500.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Ship Order"));
            meta.insert("duration".to_string(), json!("5m"));
            meta
        },
    }).unwrap();
    
    // Add edges to workflow2
    workflow2.add_edge(EdgeId::new(), pick_items, pack_order, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    }).unwrap();
    
    workflow2.add_edge(EdgeId::new(), pack_order, ship_order, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: HashMap::new(),
    }).unwrap();
    
    // Compose the workflows
    let options = CompositionOptions::default();
    let mut composed = composer.compose(&[&workflow1, &workflow2], "workflow", options).unwrap();
    
    // Add a connecting edge between the two workflows
    composed.add_edge(EdgeId::new(), process_payment, pick_items, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("label".to_string(), json!("Payment Confirmed"));
            meta
        },
    }).unwrap();
    
    println!("Composed workflow has {composed.list_nodes(} nodes and {} edges").len(),
        composed.list_edges().len()
    );
    
    // Print the complete workflow
    println!("\nComplete Order-to-Ship Workflow:");
    for (_, node_data) in composed.list_nodes() {
        if let Some(label) = node_data.metadata.get("label") {
            println!("  - {label} ({node_data.node_type})");
        }
    }
    
    println!();
}

fn conflict_resolution_example(composer: &DefaultGraphComposer) {
    println!("2. Conflict Resolution Example");
    println!("------------------------------");
    
    // Create two concept graphs with overlapping concepts
    let mut concepts1 = GraphType::new_concept(GraphId::new(), "Domain Model v1");
    let mut concepts2 = GraphType::new_concept(GraphId::new(), "Domain Model v2");
    
    let customer_id = NodeId::new();
    let order_id = NodeId::new();
    
    // Add customer concept to both graphs with different metadata
    concepts1.add_node(customer_id, NodeData {
        node_type: "entity".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("version".to_string(), json!("1.0"));
            meta.insert("attributes".to_string(), json!(["name", "email"]));
            meta.insert("description".to_string(), json!("Basic customer entity"));
            meta
        },
    }).unwrap();
    
    concepts2.add_node(customer_id, NodeData {
        node_type: "entity".to_string(),
        position: Position3D { x: 50.0, y: 50.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("version".to_string(), json!("2.0"));
            meta.insert("attributes".to_string(), json!(["name", "email", "phone", "address"]));
            meta.insert("description".to_string(), json!("Extended customer entity"));
            meta.insert("deprecated_fields".to_string(), json!(["fax"]));
            meta
        },
    }).unwrap();
    
    // Add order concept only to first graph
    concepts1.add_node(order_id, NodeData {
        node_type: "entity".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("version".to_string(), json!("1.0"));
            meta.insert("attributes".to_string(), json!(["id", "date", "total"]));
            meta
        },
    }).unwrap();
    
    // Test different conflict resolution strategies
    println!("\nTesting conflict resolution strategies:");
    
    // Strategy 1: Keep First
    let mut options = CompositionOptions::default();
    options.node_conflict_resolution = ConflictResolution::KeepFirst;
    let composed = composer.compose(&[&concepts1, &concepts2], "concept", options).unwrap();
    let customer = composed.get_node(customer_id).unwrap();
    println!("\n  KeepFirst: Customer version = {customer.metadata.get("version"}").unwrap()
    );
    
    // Strategy 2: Keep Last
    let mut options = CompositionOptions::default();
    options.node_conflict_resolution = ConflictResolution::KeepLast;
    let composed = composer.compose(&[&concepts1, &concepts2], "concept", options).unwrap();
    let customer = composed.get_node(customer_id).unwrap();
    println!("  KeepLast: Customer version = {customer.metadata.get("version"}").unwrap()
    );
    
    // Strategy 3: Merge
    let mut options = CompositionOptions::default();
    options.node_conflict_resolution = ConflictResolution::Merge;
    options.merge_metadata = true;
    let composed = composer.compose(&[&concepts1, &concepts2], "concept", options).unwrap();
    let customer = composed.get_node(customer_id).unwrap();
    println!("  Merge: Customer attributes = {customer.metadata.get("attributes"}").unwrap()
    );
    println!("         Position = ({:.1}, {:.1})", 
        customer.position.x, customer.position.y
    );
    
    println!();
}

fn cross_type_composition_example(composer: &DefaultGraphComposer) {
    println!("3. Cross-Type Composition Example");
    println!("---------------------------------");
    
    // Create graphs of different types
    let mut context = GraphType::new_context(GraphId::new(), "Business Context");
    let mut workflow = GraphType::new_workflow(GraphId::new(), "Business Process");
    let mut concepts = GraphType::new_concept(GraphId::new(), "Domain Concepts");
    
    // Add business entities to context
    let customer_service = NodeId::new();
    let order_management = NodeId::new();
    
    context.add_node(customer_service, NodeData {
        node_type: "bounded_context".to_string(),
        position: Position3D { x: 0.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Customer Service"));
            meta.insert("team".to_string(), json!("CS Team"));
            meta
        },
    }).unwrap();
    
    context.add_node(order_management, NodeData {
        node_type: "bounded_context".to_string(),
        position: Position3D { x: 200.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Order Management"));
            meta.insert("team".to_string(), json!("OM Team"));
            meta
        },
    }).unwrap();
    
    // Add workflow steps
    let create_ticket = NodeId::new();
    let resolve_issue = NodeId::new();
    
    workflow.add_node(create_ticket, NodeData {
        node_type: "task".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Create Support Ticket"));
            meta.insert("context".to_string(), json!("Customer Service"));
            meta
        },
    }).unwrap();
    
    workflow.add_node(resolve_issue, NodeData {
        node_type: "task".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Resolve Issue"));
            meta.insert("context".to_string(), json!("Customer Service"));
            meta
        },
    }).unwrap();
    
    // Add domain concepts
    let ticket_concept = NodeId::new();
    
    concepts.add_node(ticket_concept, NodeData {
        node_type: "concept".to_string(),
        position: Position3D { x: 50.0, y: -100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Support Ticket"));
            meta.insert("properties".to_string(), json!(["id", "status", "priority"]));
            meta
        },
    }).unwrap();
    
    // Compose all three graphs into a unified view
    let options = CompositionOptions::default();
    let unified = composer.compose(&[&context, &workflow, &concepts], "context", options).unwrap();
    
    println!("Unified graph contains:");
    println!("  - {unified.find_nodes_by_type("bounded_context"} bounded contexts").len()
    );
    println!("  - {unified.find_nodes_by_type("task"} workflow tasks").len()
    );
    println!("  - {unified.find_nodes_by_type("concept"} domain concepts").len()
    );
    
    println!();
}

fn custom_mapping_example(composer: &DefaultGraphComposer) {
    println!("4. Custom Mapping Example");
    println!("-------------------------");
    
    // Create two graphs that would have ID conflicts
    let mut team_a_graph = GraphType::new_workflow(GraphId::new(), "Team A Workflow");
    let mut team_b_graph = GraphType::new_workflow(GraphId::new(), "Team B Workflow");
    
    // Both teams use simple IDs that might conflict
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();
    
    // Team A's workflow
    team_a_graph.add_node(node1, NodeData {
        node_type: "task".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Team A: Data Collection"));
            meta.insert("team".to_string(), json!("A"));
            meta
        },
    }).unwrap();
    
    team_a_graph.add_node(node2, NodeData {
        node_type: "task".to_string(),
        position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Team A: Data Processing"));
            meta.insert("team".to_string(), json!("A"));
            meta
        },
    }).unwrap();
    
    // Team B's workflow (same IDs!)
    team_b_graph.add_node(node1, NodeData {
        node_type: "task".to_string(),
        position: Position3D { x: 0.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Team B: Report Generation"));
            meta.insert("team".to_string(), json!("B"));
            meta
        },
    }).unwrap();
    
    team_b_graph.add_node(node2, NodeData {
        node_type: "task".to_string(),
        position: Position3D { x: 100.0, y: 100.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Team B: Report Distribution"));
            meta.insert("team".to_string(), json!("B"));
            meta
        },
    }).unwrap();
    
    // Set up custom mappings to avoid conflicts
    let mut options = CompositionOptions::default();
    
    // Map Team B's nodes to new IDs
    let mut team_b_mappings = HashMap::new();
    team_b_mappings.insert(node1, node3);
    team_b_mappings.insert(node2, NodeId::new());
    
    options.node_id_mappings.insert(team_b_graph.graph_id(), team_b_mappings);
    
    // Compose with custom mappings
    let composed = composer.compose(&[&team_a_graph, &team_b_graph], "workflow", options).unwrap();
    
    println!("After composition with custom mappings:");
    for (_node_id, node_data) in composed.list_nodes() {
        if let Some(name) = node_data.metadata.get("name") {
            println!("  - Node {node_data.metadata.get("team"} -> {}").unwrap_or(&json!("?")).as_str().unwrap(),
                name
            );
        }
    }
    
    println!("\nTotal nodes: {composed.list_nodes(} (no conflicts!)").len());
} 