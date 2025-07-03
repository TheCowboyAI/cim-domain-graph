//! Graph Transformation Demo
//! 
//! This example demonstrates how to transform graphs between different types
//! (Context, Concept, Workflow, IPLD) while preserving data integrity.

use cim_domain_graph::{
    GraphId, NodeId,
    abstraction::{
        GraphType, GraphImplementation, NodeData, EdgeData, Position3D,
        DefaultGraphTransformer, GraphTransformer, TransformationOptions,
    },
};
use std::collections::HashMap;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph Transformation Demo ===\n");
    
    // Create a transformer
    let transformer = DefaultGraphTransformer::new();
    
    // Demo 1: Create a workflow graph and transform it to a concept graph
    println!("1. Creating a Workflow Graph and transforming to Concept Graph...");
    let workflow_graph = create_sample_workflow_graph()?;
    demo_workflow_to_concept_transformation(&transformer, &workflow_graph)?;
    
    // Demo 2: Create a context graph and transform it to multiple types
    println!("\n2. Creating a Context Graph and transforming to multiple types...");
    let context_graph = create_sample_context_graph()?;
    demo_context_transformations(&transformer, &context_graph)?;
    
    // Demo 3: Transform with custom mappings
    println!("\n3. Transforming with custom node/edge type mappings...");
    demo_custom_mapping_transformation(&transformer, &workflow_graph)?;
    
    // Demo 4: Preview data loss
    println!("\n4. Previewing potential data loss in transformations...");
    demo_data_loss_preview(&transformer)?;
    
    println!("\n=== Demo Complete ===");
    Ok(())
}

fn create_sample_workflow_graph() -> Result<GraphType, Box<dyn std::error::Error>> {
    let graph_id = GraphId::new();
    let mut graph = GraphType::new_workflow(graph_id, "Order Processing Workflow");
    
    // Add workflow steps
    let start_node = NodeId::new();
    graph.add_node(start_node, NodeData {
        node_type: "start".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Receive Order"));
            meta.insert("step_type".to_string(), json!("entry"));
            meta
        },
    })?;
    
    let validate_node = NodeId::new();
    graph.add_node(validate_node, NodeData {
        node_type: "process".to_string(),
        position: Position3D { x: 20.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Validate Order"));
            meta.insert("step_type".to_string(), json!("validation"));
            meta.insert("execution_time".to_string(), json!(30));
            meta
        },
    })?;
    
    let decision_node = NodeId::new();
    graph.add_node(decision_node, NodeData {
        node_type: "decision".to_string(),
        position: Position3D { x: 40.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Check Inventory"));
            meta.insert("step_type".to_string(), json!("decision"));
            meta
        },
    })?;
    
    // Add edges
    let edge1 = cim_domain::EdgeId::new();
    graph.add_edge(edge1, start_node, validate_node, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("flow_type".to_string(), json!("normal"));
            meta
        },
    })?;
    
    let edge2 = cim_domain::EdgeId::new();
    graph.add_edge(edge2, validate_node, decision_node, EdgeData {
        edge_type: "sequence".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("flow_type".to_string(), json!("conditional"));
            meta.insert("condition".to_string(), json!("order.isValid"));
            meta
        },
    })?;
    
    Ok(graph)
}

fn create_sample_context_graph() -> Result<GraphType, Box<dyn std::error::Error>> {
    let graph_id = GraphId::new();
    let mut graph = GraphType::new_context(graph_id, "User Context");
    
    // Add context nodes
    let user_node = NodeId::new();
    graph.add_node(user_node, NodeData {
        node_type: "user".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Alice"));
            meta.insert("role".to_string(), json!("admin"));
            meta.insert("context_type".to_string(), json!("identity"));
            meta
        },
    })?;
    
    let permission_node = NodeId::new();
    graph.add_node(permission_node, NodeData {
        node_type: "permission".to_string(),
        position: Position3D { x: 20.0, y: 10.0, z: 0.0 },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("read_write"));
            meta.insert("scope".to_string(), json!("global"));
            meta.insert("context_type".to_string(), json!("authorization"));
            meta
        },
    })?;
    
    // Add relationship
    let edge = cim_domain::EdgeId::new();
    graph.add_edge(edge, user_node, permission_node, EdgeData {
        edge_type: "has_permission".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("granted_date".to_string(), json!("2025-01-27"));
            meta.insert("relationship_context".to_string(), json!("security"));
            meta
        },
    })?;
    
    Ok(graph)
}

fn demo_workflow_to_concept_transformation(
    transformer: &DefaultGraphTransformer,
    workflow_graph: &GraphType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Original workflow graph:");
    print_graph_summary(workflow_graph);
    
    // Transform with default options
    let options = TransformationOptions {
        preserve_metadata: true,
        allow_data_loss: false,
        ..Default::default()
    };
    
    // First check for data loss
    let data_loss = transformer.preview_data_loss(workflow_graph, "concept");
    if !data_loss.is_empty() {
        println!("   ⚠️  Warning: Potential data loss detected:");
        for warning in &data_loss {
            println!("      - {warning}");
        }
        
        // Transform anyway with data loss allowed
        let mut options_with_loss = options.clone();
        options_with_loss.allow_data_loss = true;
        
        let concept_graph = transformer.transform(workflow_graph, "concept", options_with_loss)?;
        println!("\n   Transformed to concept graph (with data loss):");
        print_graph_summary(&concept_graph);
    } else {
        let concept_graph = transformer.transform(workflow_graph, "concept", options)?;
        println!("\n   Transformed to concept graph:");
        print_graph_summary(&concept_graph);
    }
    
    Ok(())
}

fn demo_context_transformations(
    transformer: &DefaultGraphTransformer,
    context_graph: &GraphType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Original context graph:");
    print_graph_summary(context_graph);
    
    let options = TransformationOptions {
        preserve_metadata: true,
        allow_data_loss: true,
        ..Default::default()
    };
    
    // Transform to workflow
    let workflow_graph = transformer.transform(context_graph, "workflow", options.clone())?;
    println!("\n   Transformed to workflow graph:");
    print_graph_summary(&workflow_graph);
    
    // Transform to IPLD
    let ipld_graph = transformer.transform(context_graph, "ipld", options.clone())?;
    println!("\n   Transformed to IPLD graph:");
    print_graph_summary(&ipld_graph);
    
    Ok(())
}

fn demo_custom_mapping_transformation(
    transformer: &DefaultGraphTransformer,
    workflow_graph: &GraphType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Using custom mappings to transform workflow to context graph...");
    
    let mut options = TransformationOptions {
        preserve_metadata: true,
        allow_data_loss: true,
        ..Default::default()
    };
    
    // Add custom node type mappings
    options.node_type_mappings.insert("start".to_string(), "entry_point".to_string());
    options.node_type_mappings.insert("process".to_string(), "action".to_string());
    options.node_type_mappings.insert("decision".to_string(), "branch".to_string());
    
    // Add custom edge type mappings
    options.edge_type_mappings.insert("sequence".to_string(), "follows".to_string());
    
    // Add additional metadata
    options.additional_metadata.insert("transformation_date".to_string(), json!("2025-01-27"));
    options.additional_metadata.insert("transformed_by".to_string(), json!("demo_system"));
    
    let context_graph = transformer.transform(workflow_graph, "context", options)?;
    println!("\n   Transformed with custom mappings:");
    print_graph_summary(&context_graph);
    
    // Show the mapped node types
    println!("\n   Node type mappings applied:");
    for (node_id, node_data) in context_graph.list_nodes() {
        println!("      - Node {node_id}: {node_data.node_type} (transformed from workflow node)");
    }
    
    Ok(())
}

fn demo_data_loss_preview(
    transformer: &DefaultGraphTransformer,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a concept graph with semantic embeddings
    let graph_id = GraphId::new();
    let mut concept_graph = GraphType::new_concept(graph_id, "ML Concepts");
    
    let node = NodeId::new();
    concept_graph.add_node(node, NodeData {
        node_type: "concept".to_string(),
        position: Position3D::default(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!("Neural Network"));
            meta.insert("semantic_embedding".to_string(), json!([0.1, 0.2, 0.3, 0.4, 0.5]));
            meta.insert("conceptual_coordinates".to_string(), json!([1.0, 2.0, 3.0]));
            meta
        },
    })?;
    
    // Preview transformations
    println!("   Checking data loss for various transformations:");
    
    let transformations = vec![
        ("concept", "workflow"),
        ("concept", "context"),
        ("concept", "ipld"),
    ];
    
    for (from, to) in transformations {
        let warnings = transformer.preview_data_loss(&concept_graph, to);
        if warnings.is_empty() {
            println!("   ✅ {from} → {to}: No data loss");
        } else {
            println!("   ⚠️  {from} → {to}: {warnings.join(", "}"));
        }
    }
    
    Ok(())
}

fn print_graph_summary(graph: &GraphType) {
    let metadata = graph.get_metadata();
    let nodes = graph.list_nodes();
    let edges = graph.list_edges();
    
    println!("      Name: {metadata.name}");
    println!("      Nodes: {nodes.len(} total"));
    for (_, node_data) in nodes.iter().take(3) {
        println!("        - Type: {node_data.node_type}, Metadata keys: {:?}", node_data.metadata.keys().collect::<Vec<_>>());
    }
    if nodes.len() > 3 {
        println!("        ... and {nodes.len(} more") - 3);
    }
    
    println!("      Edges: {edges.len(} total"));
    for (_, edge_data, _, _) in edges.iter().take(2) {
        println!("        - Type: {edge_data.edge_type}, Metadata keys: {:?}", edge_data.metadata.keys().collect::<Vec<_>>());
    }
    if edges.len() > 2 {
        println!("        ... and {edges.len(} more") - 2);
    }
} 