//! Graph Abstraction Layer Demo
//! 
//! This example demonstrates how the graph abstraction layer allows working with
//! different graph implementations (Context, Concept, Workflow, IPLD) through a
//! unified interface.

use cim_domain_graph::{
    GraphId, NodeId,
    abstraction::Position3D,
    handlers::{AbstractGraphCommandHandler, InMemoryAbstractGraphRepository},
    commands::GraphCommand,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph Abstraction Layer Demo ===\n");
    
    // Create repository and handler
    let repository = Arc::new(InMemoryAbstractGraphRepository::new());
    let handler = AbstractGraphCommandHandler::new(repository.clone());
    
    // Demo 1: Create a Context Graph
    println!("1. Creating a Context Graph...");
    let context_graph = create_context_graph(&handler).await?;
    println!("   Created context graph with ID: {}", context_graph);
    
    // Demo 2: Create a Workflow Graph
    println!("\n2. Creating a Workflow Graph...");
    let workflow_graph = create_workflow_graph(&handler).await?;
    println!("   Created workflow graph with ID: {}", workflow_graph);
    
    // Demo 3: Create a Concept Graph
    println!("\n3. Creating a Concept Graph...");
    let concept_graph = create_concept_graph(&handler).await?;
    println!("   Created concept graph with ID: {}", concept_graph);
    
    // Demo 4: Create an IPLD Graph
    println!("\n4. Creating an IPLD Graph...");
    let ipld_graph = create_ipld_graph(&handler).await?;
    println!("   Created IPLD graph with ID: {}", ipld_graph);
    
    // Demo 5: Show graph statistics
    println!("\n5. Graph Statistics:");
    show_graph_stats(&repository, context_graph).await?;
    show_graph_stats(&repository, workflow_graph).await?;
    show_graph_stats(&repository, concept_graph).await?;
    show_graph_stats(&repository, ipld_graph).await?;
    
    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn create_context_graph(handler: &AbstractGraphCommandHandler) -> Result<GraphId, Box<dyn std::error::Error>> {
    // Create the graph
    let mut metadata = HashMap::new();
    metadata.insert("graph_type".to_string(), serde_json::Value::String("context".to_string()));
    
    let create_command = GraphCommand::CreateGraph {
        name: "User Context".to_string(),
        description: "A context graph showing user relationships".to_string(),
        metadata,
    };
    
    let events = handler.process_graph_command(
        create_command,
        &cim_domain::CommandEnvelope::new(
            GraphCommand::CreateGraph {
                name: "User Context".to_string(),
                description: "A context graph showing user relationships".to_string(),
                metadata: HashMap::new(),
            },
            "demo".to_string()
        )
    ).await?;
    
    let graph_id = match &events[0] {
        cim_domain_graph::domain_events::GraphDomainEvent::GraphCreated(event) => event.graph_id,
        _ => panic!("Expected GraphCreated event"),
    };
    
    // Add some nodes
    let node_ids = add_context_nodes(handler, graph_id).await?;
    
    // Add edges
    add_context_edges(handler, graph_id, &node_ids).await?;
    
    Ok(graph_id)
}

async fn create_workflow_graph(handler: &AbstractGraphCommandHandler) -> Result<GraphId, Box<dyn std::error::Error>> {
    // Create the graph
    let mut metadata = HashMap::new();
    metadata.insert("graph_type".to_string(), serde_json::Value::String("workflow".to_string()));
    
    let create_command = GraphCommand::CreateGraph {
        name: "Order Processing Workflow".to_string(),
        description: "A workflow for processing customer orders".to_string(),
        metadata,
    };
    
    let events = handler.process_graph_command(
        create_command,
        &cim_domain::CommandEnvelope::new(
            GraphCommand::CreateGraph {
                name: "Order Processing Workflow".to_string(),
                description: "A workflow for processing customer orders".to_string(),
                metadata: HashMap::new(),
            },
            "demo".to_string()
        )
    ).await?;
    
    let graph_id = match &events[0] {
        cim_domain_graph::domain_events::GraphDomainEvent::GraphCreated(event) => event.graph_id,
        _ => panic!("Expected GraphCreated event"),
    };
    
    // Add workflow steps
    let step_ids = add_workflow_steps(handler, graph_id).await?;
    
    // Connect workflow steps
    connect_workflow_steps(handler, graph_id, &step_ids).await?;
    
    Ok(graph_id)
}

async fn create_concept_graph(handler: &AbstractGraphCommandHandler) -> Result<GraphId, Box<dyn std::error::Error>> {
    // Create the graph
    let mut metadata = HashMap::new();
    metadata.insert("graph_type".to_string(), serde_json::Value::String("concept".to_string()));
    
    let create_command = GraphCommand::CreateGraph {
        name: "Machine Learning Concepts".to_string(),
        description: "A conceptual graph of ML concepts and their relationships".to_string(),
        metadata,
    };
    
    let events = handler.process_graph_command(
        create_command,
        &cim_domain::CommandEnvelope::new(
            GraphCommand::CreateGraph {
                name: "Machine Learning Concepts".to_string(),
                description: "A conceptual graph of ML concepts and their relationships".to_string(),
                metadata: HashMap::new(),
            },
            "demo".to_string()
        )
    ).await?;
    
    let graph_id = match &events[0] {
        cim_domain_graph::domain_events::GraphDomainEvent::GraphCreated(event) => event.graph_id,
        _ => panic!("Expected GraphCreated event"),
    };
    
    // Add concepts
    let concept_ids = add_concepts(handler, graph_id).await?;
    
    // Add semantic relationships
    add_semantic_relationships(handler, graph_id, &concept_ids).await?;
    
    Ok(graph_id)
}

async fn create_ipld_graph(handler: &AbstractGraphCommandHandler) -> Result<GraphId, Box<dyn std::error::Error>> {
    // Create the graph
    let mut metadata = HashMap::new();
    metadata.insert("graph_type".to_string(), serde_json::Value::String("ipld".to_string()));
    
    let create_command = GraphCommand::CreateGraph {
        name: "Event Chain".to_string(),
        description: "An IPLD graph showing event causality".to_string(),
        metadata,
    };
    
    let events = handler.process_graph_command(
        create_command,
        &cim_domain::CommandEnvelope::new(
            GraphCommand::CreateGraph {
                name: "Event Chain".to_string(),
                description: "An IPLD graph showing event causality".to_string(),
                metadata: HashMap::new(),
            },
            "demo".to_string()
        )
    ).await?;
    
    let graph_id = match &events[0] {
        cim_domain_graph::domain_events::GraphDomainEvent::GraphCreated(event) => event.graph_id,
        _ => panic!("Expected GraphCreated event"),
    };
    
    // Add events as nodes
    let event_ids = add_event_nodes(handler, graph_id).await?;
    
    // Add causal relationships
    add_causal_relationships(handler, graph_id, &event_ids).await?;
    
    Ok(graph_id)
}

async fn add_context_nodes(handler: &AbstractGraphCommandHandler, graph_id: GraphId) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let mut node_ids = Vec::new();
    
    let nodes = vec![
        ("user", "Alice", Position3D { x: 0.0, y: 0.0, z: 0.0 }),
        ("user", "Bob", Position3D { x: 10.0, y: 0.0, z: 0.0 }),
        ("role", "Admin", Position3D { x: 0.0, y: 10.0, z: 0.0 }),
        ("permission", "Read", Position3D { x: 10.0, y: 10.0, z: 0.0 }),
    ];
    
    for (node_type, name, position) in nodes {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        metadata.insert("position".to_string(), serde_json::json!({
            "x": position.x,
            "y": position.y,
            "z": position.z
        }));
        
        let command = GraphCommand::AddNode {
            graph_id,
            node_type: node_type.to_string(),
            metadata,
        };
        
        let events = handler.process_graph_command(
            command,
            &cim_domain::CommandEnvelope::new(
                GraphCommand::AddNode {
                    graph_id,
                    node_type: node_type.to_string(),
                    metadata: HashMap::new(),
                },
                "demo".to_string()
            )
        ).await?;
        
        if let cim_domain_graph::domain_events::GraphDomainEvent::NodeAdded(event) = &events[0] {
            node_ids.push(event.node_id);
        }
    }
    
    Ok(node_ids)
}

async fn add_context_edges(handler: &AbstractGraphCommandHandler, graph_id: GraphId, node_ids: &[NodeId]) -> Result<(), Box<dyn std::error::Error>> {
    // Alice has Admin role
    add_edge(handler, graph_id, node_ids[0], node_ids[2], "has_role").await?;
    // Admin role has Read permission
    add_edge(handler, graph_id, node_ids[2], node_ids[3], "has_permission").await?;
    // Bob has Read permission directly
    add_edge(handler, graph_id, node_ids[1], node_ids[3], "has_permission").await?;
    
    Ok(())
}

async fn add_workflow_steps(handler: &AbstractGraphCommandHandler, graph_id: GraphId) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let mut step_ids = Vec::new();
    
    let steps = vec![
        ("start", "Receive Order", Position3D { x: 0.0, y: 0.0, z: 0.0 }),
        ("manual", "Validate Order", Position3D { x: 20.0, y: 0.0, z: 0.0 }),
        ("automated", "Check Inventory", Position3D { x: 40.0, y: 0.0, z: 0.0 }),
        ("decision", "In Stock?", Position3D { x: 60.0, y: 0.0, z: 0.0 }),
        ("automated", "Process Payment", Position3D { x: 80.0, y: -10.0, z: 0.0 }),
        ("manual", "Backorder", Position3D { x: 80.0, y: 10.0, z: 0.0 }),
        ("end", "Complete", Position3D { x: 100.0, y: 0.0, z: 0.0 }),
    ];
    
    for (step_type, name, position) in steps {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        metadata.insert("step_type".to_string(), serde_json::Value::String(step_type.to_string()));
        metadata.insert("position".to_string(), serde_json::json!({
            "x": position.x,
            "y": position.y,
            "z": position.z
        }));
        
        let command = GraphCommand::AddNode {
            graph_id,
            node_type: step_type.to_string(),
            metadata,
        };
        
        let events = handler.process_graph_command(
            command,
            &cim_domain::CommandEnvelope::new(
                GraphCommand::AddNode {
                    graph_id,
                    node_type: step_type.to_string(),
                    metadata: HashMap::new(),
                },
                "demo".to_string()
            )
        ).await?;
        
        if let cim_domain_graph::domain_events::GraphDomainEvent::NodeAdded(event) = &events[0] {
            step_ids.push(event.node_id);
        }
    }
    
    Ok(step_ids)
}

async fn connect_workflow_steps(handler: &AbstractGraphCommandHandler, graph_id: GraphId, step_ids: &[NodeId]) -> Result<(), Box<dyn std::error::Error>> {
    // Linear flow
    add_edge(handler, graph_id, step_ids[0], step_ids[1], "next").await?;
    add_edge(handler, graph_id, step_ids[1], step_ids[2], "next").await?;
    add_edge(handler, graph_id, step_ids[2], step_ids[3], "next").await?;
    
    // Decision branches
    add_edge(handler, graph_id, step_ids[3], step_ids[4], "yes").await?;
    add_edge(handler, graph_id, step_ids[3], step_ids[5], "no").await?;
    
    // Both branches lead to complete
    add_edge(handler, graph_id, step_ids[4], step_ids[6], "next").await?;
    add_edge(handler, graph_id, step_ids[5], step_ids[6], "next").await?;
    
    Ok(())
}

async fn add_concepts(handler: &AbstractGraphCommandHandler, graph_id: GraphId) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let mut concept_ids = Vec::new();
    
    let concepts = vec![
        ("Machine Learning", Position3D { x: 0.0, y: 0.0, z: 0.0 }),
        ("Neural Networks", Position3D { x: 20.0, y: -10.0, z: 0.0 }),
        ("Deep Learning", Position3D { x: 40.0, y: -10.0, z: 0.0 }),
        ("Supervised Learning", Position3D { x: 20.0, y: 10.0, z: 0.0 }),
        ("Classification", Position3D { x: 40.0, y: 10.0, z: 0.0 }),
    ];
    
    for (name, position) in concepts {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        metadata.insert("position".to_string(), serde_json::json!({
            "x": position.x,
            "y": position.y,
            "z": position.z
        }));
        
        let command = GraphCommand::AddNode {
            graph_id,
            node_type: "concept".to_string(),
            metadata,
        };
        
        let events = handler.process_graph_command(
            command,
            &cim_domain::CommandEnvelope::new(
                GraphCommand::AddNode {
                    graph_id,
                    node_type: "concept".to_string(),
                    metadata: HashMap::new(),
                },
                "demo".to_string()
            )
        ).await?;
        
        if let cim_domain_graph::domain_events::GraphDomainEvent::NodeAdded(event) = &events[0] {
            concept_ids.push(event.node_id);
        }
    }
    
    Ok(concept_ids)
}

async fn add_semantic_relationships(handler: &AbstractGraphCommandHandler, graph_id: GraphId, concept_ids: &[NodeId]) -> Result<(), Box<dyn std::error::Error>> {
    // ML is parent of Neural Networks and Supervised Learning
    add_edge(handler, graph_id, concept_ids[0], concept_ids[1], "includes").await?;
    add_edge(handler, graph_id, concept_ids[0], concept_ids[3], "includes").await?;
    
    // Neural Networks is parent of Deep Learning
    add_edge(handler, graph_id, concept_ids[1], concept_ids[2], "specializes").await?;
    
    // Supervised Learning includes Classification
    add_edge(handler, graph_id, concept_ids[3], concept_ids[4], "includes").await?;
    
    // Deep Learning is related to Classification
    add_edge(handler, graph_id, concept_ids[2], concept_ids[4], "related_to").await?;
    
    Ok(())
}

async fn add_event_nodes(handler: &AbstractGraphCommandHandler, graph_id: GraphId) -> Result<Vec<NodeId>, Box<dyn std::error::Error>> {
    let mut event_ids = Vec::new();
    
    let events = vec![
        ("UserCreated", "user-123", Position3D { x: 0.0, y: 0.0, z: 0.0 }),
        ("ProfileUpdated", "user-123", Position3D { x: 20.0, y: 0.0, z: 0.0 }),
        ("OrderPlaced", "order-456", Position3D { x: 40.0, y: 0.0, z: 0.0 }),
        ("PaymentProcessed", "payment-789", Position3D { x: 60.0, y: 0.0, z: 0.0 }),
    ];
    
    for (event_type, entity_id, position) in events {
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), serde_json::Value::String(event_type.to_string()));
        metadata.insert("entity_id".to_string(), serde_json::Value::String(entity_id.to_string()));
        metadata.insert("timestamp".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        metadata.insert("position".to_string(), serde_json::json!({
            "x": position.x,
            "y": position.y,
            "z": position.z
        }));
        
        let command = GraphCommand::AddNode {
            graph_id,
            node_type: "event".to_string(),
            metadata,
        };
        
        let events = handler.process_graph_command(
            command,
            &cim_domain::CommandEnvelope::new(
                GraphCommand::AddNode {
                    graph_id,
                    node_type: "event".to_string(),
                    metadata: HashMap::new(),
                },
                "demo".to_string()
            )
        ).await?;
        
        if let cim_domain_graph::domain_events::GraphDomainEvent::NodeAdded(event) = &events[0] {
            event_ids.push(event.node_id);
        }
    }
    
    Ok(event_ids)
}

async fn add_causal_relationships(handler: &AbstractGraphCommandHandler, graph_id: GraphId, event_ids: &[NodeId]) -> Result<(), Box<dyn std::error::Error>> {
    // User creation causes profile update
    add_edge(handler, graph_id, event_ids[0], event_ids[1], "causes").await?;
    // Profile update enables order placement
    add_edge(handler, graph_id, event_ids[1], event_ids[2], "enables").await?;
    // Order placement triggers payment processing
    add_edge(handler, graph_id, event_ids[2], event_ids[3], "triggers").await?;
    
    Ok(())
}

async fn add_edge(handler: &AbstractGraphCommandHandler, graph_id: GraphId, source: NodeId, target: NodeId, edge_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let command = GraphCommand::AddEdge {
        graph_id,
        source_id: source,
        target_id: target,
        edge_type: edge_type.to_string(),
        metadata: HashMap::new(),
    };
    
    handler.process_graph_command(
        command,
        &cim_domain::CommandEnvelope::new(
            GraphCommand::AddEdge {
                graph_id,
                source_id: source,
                target_id: target,
                edge_type: edge_type.to_string(),
                metadata: HashMap::new(),
            },
            "demo".to_string()
        )
    ).await?;
    
    Ok(())
}

async fn show_graph_stats(repository: &Arc<InMemoryAbstractGraphRepository>, graph_id: GraphId) -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_graph::handlers::AbstractGraphRepository;
    
    let graph = repository.load(graph_id).await?;
    let nodes = graph.list_nodes();
    let edges = graph.list_edges();
    
    println!("   Graph '{}': {} nodes, {} edges", graph.name(), nodes.len(), edges.len());
    
    Ok(())
} 