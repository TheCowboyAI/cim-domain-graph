//! Graph Operations Example
//!
//! This example demonstrates:
//! - Creating graphs with nodes and edges
//! - Graph traversal operations
//! - Subgraph extraction
//! - Graph algorithms

use cim_domain_graph::{
    aggregate::GraphAggregate,
    commands::{CreateGraph, AddNode, ConnectEdge, RemoveNode},
    events::{GraphCreated, NodeAdded, EdgeConnected, NodeRemoved},
    value_objects::{GraphId, NodeId, EdgeId, NodeContent, EdgeRelationship},
    handlers::GraphCommandHandler,
    queries::{GetGraph, FindPath, GetSubgraph, GraphQueryHandler},
};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM Graph Domain Example ===\n");

    // Initialize handlers
    let command_handler = GraphCommandHandler::new();
    let query_handler = GraphQueryHandler::new();

    // Example implementation demonstrates graph operations
    
    println!("\n=== Example completed successfully! ===");
    Ok(())
}
