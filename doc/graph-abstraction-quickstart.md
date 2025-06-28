# Graph Abstraction Layer Quick Start Guide

## Installation

Add the graph domain to your `Cargo.toml`:

```toml
[dependencies]
cim-domain-graph = { path = "../cim-domain-graph" }
```

## Basic Usage

### 1. Creating a Graph

```rust
use cim_domain_graph::abstraction::{GraphType, NodeData, EdgeData, Position3D};
use cim_domain::{GraphId, NodeId, EdgeId};

// Create a context graph
let graph_id = GraphId::new();
let mut graph = GraphType::new_context(graph_id, "My Knowledge Graph");

// Add nodes
let person_node = NodeId::new();
let company_node = NodeId::new();

graph.add_node(person_node, NodeData {
    node_type: "person".to_string(),
    position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("name".to_string(), json!("Alice"));
        meta.insert("role".to_string(), json!("Engineer"));
        meta
    },
})?;

graph.add_node(company_node, NodeData {
    node_type: "company".to_string(),
    position: Position3D { x: 100.0, y: 0.0, z: 0.0 },
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("name".to_string(), json!("TechCorp"));
        meta
    },
})?;

// Connect nodes
let edge_id = EdgeId::new();
graph.add_edge(edge_id, person_node, company_node, EdgeData {
    edge_type: "works_at".to_string(),
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("since".to_string(), json!("2020"));
        meta
    },
})?;
```

### 2. Transforming Graphs

```rust
use cim_domain_graph::abstraction::{
    DefaultGraphTransformer, GraphTransformer, TransformationOptions
};

// Create a transformer
let transformer = DefaultGraphTransformer::new();

// Transform context graph to workflow graph
let workflow_graph = transformer.transform(
    &graph,
    "workflow",
    TransformationOptions::default()
)?;

// Custom transformation with type mappings
let mut options = TransformationOptions::default();
options.type_mappings.insert(
    "person".to_string(),
    "actor".to_string()
);
options.type_mappings.insert(
    "company".to_string(),
    "system".to_string()
);

let custom_workflow = transformer.transform(
    &graph,
    "workflow",
    options
)?;
```

### 3. Composing Multiple Graphs

```rust
use cim_domain_graph::abstraction::{
    DefaultGraphComposer, GraphComposer, CompositionOptions, ConflictResolution
};

// Create multiple graphs
let graph1 = GraphType::new_context(GraphId::new(), "Graph 1");
let graph2 = GraphType::new_concept(GraphId::new(), "Graph 2");

// Create composer
let composer = DefaultGraphComposer::new();

// Compose with default options (merge conflicts)
let composed = composer.compose(
    &[&graph1, &graph2],
    "context",
    CompositionOptions::default()
)?;

// Compose with custom conflict resolution
let mut options = CompositionOptions::default();
options.conflict_resolution = ConflictResolution::KeepLast;
options.validate_edges = true;

let composed_custom = composer.compose(
    &[&graph1, &graph2],
    "context",
    options
)?;
```

### 4. Integrating with Bevy ECS

```rust
use bevy_app::App;
use cim_domain_graph::abstraction::{GraphAbstractionPlugin, GraphAbstractionLayer};

// Create handlers
let repository = Arc::new(UnifiedGraphRepositoryImpl::new(event_store));
let command_handler = Arc::new(UnifiedGraphCommandHandler::new(repository.clone()));
let query_handler = Arc::new(AbstractGraphQueryHandler::new(repository));

// Add to Bevy app
let mut app = App::new();
app.add_plugins(GraphAbstractionPlugin {
    command_handler,
    query_handler,
});

// Access abstraction layer in systems
fn my_system(abstraction: Res<GraphAbstractionLayer>) {
    // Use the abstraction layer
    let graph = abstraction.get_or_create_graph(
        graph_id,
        GraphType::Context,
        "My Graph"
    ).await?;
}
```

## Common Patterns

### Pattern 1: Knowledge Graph Construction

```rust
// Create a knowledge graph with concepts and relationships
let mut knowledge_graph = GraphType::new_concept(GraphId::new(), "Domain Knowledge");

// Add concept nodes
let concepts = vec![
    ("database", "technology"),
    ("sql", "language"),
    ("postgres", "database"),
    ("mysql", "database"),
];

let mut concept_nodes = HashMap::new();
for (name, category) in concepts {
    let node_id = NodeId::new();
    knowledge_graph.add_node(node_id, NodeData {
        node_type: category.to_string(),
        position: Position3D::default(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!(name));
            meta
        },
    })?;
    concept_nodes.insert(name, node_id);
}

// Add relationships
knowledge_graph.add_edge(
    EdgeId::new(),
    concept_nodes["sql"],
    concept_nodes["postgres"],
    EdgeData {
        edge_type: "used_by".to_string(),
        metadata: HashMap::new(),
    }
)?;
```

### Pattern 2: Workflow Definition

```rust
// Create a workflow graph
let mut workflow = GraphType::new_workflow(GraphId::new(), "Data Pipeline");

// Add workflow steps
let steps = vec![
    ("extract", "Extract data from source"),
    ("transform", "Transform and clean data"),
    ("load", "Load into destination"),
];

let mut step_nodes = HashMap::new();
for (i, (id, description)) in steps.iter().enumerate() {
    let node_id = NodeId::new();
    workflow.add_node(node_id, NodeData {
        node_type: "step".to_string(),
        position: Position3D { 
            x: i as f32 * 100.0, 
            y: 0.0, 
            z: 0.0 
        },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), json!(id));
            meta.insert("description".to_string(), json!(description));
            meta.insert("order".to_string(), json!(i));
            meta
        },
    })?;
    step_nodes.insert(*id, node_id);
}

// Connect steps in sequence
workflow.add_edge(
    EdgeId::new(),
    step_nodes["extract"],
    step_nodes["transform"],
    EdgeData {
        edge_type: "dependency".to_string(),
        metadata: HashMap::new(),
    }
)?;
```

### Pattern 3: Cross-Type Composition

```rust
// Compose different graph types into unified view
let knowledge = GraphType::new_concept(GraphId::new(), "Knowledge");
let process = GraphType::new_workflow(GraphId::new(), "Process");

// Add ID prefixes to avoid conflicts
let mut options = CompositionOptions::default();
options.id_mappings.insert(
    knowledge.graph_id(),
    "knowledge_".to_string()
);
options.id_mappings.insert(
    process.graph_id(),
    "process_".to_string()
);

let unified = composer.compose(
    &[&knowledge, &process],
    "context",
    options
)?;
```

## Error Handling

```rust
use cim_domain_graph::abstraction::GraphOperationError;

match graph.add_node(node_id, node_data) {
    Ok(()) => println!("Node added successfully"),
    Err(GraphOperationError::NodeAlreadyExists) => {
        println!("Node already exists, updating instead");
        // Handle update logic
    }
    Err(GraphOperationError::InvalidNodeData(msg)) => {
        eprintln!("Invalid node data: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {:?}", e);
    }
}
```

## Performance Tips

1. **Batch Operations**: Add multiple nodes/edges before transforming
2. **Reuse Transformers**: Create once, use many times
3. **Cache Transformed Graphs**: Store results of expensive transformations
4. **Use Appropriate Graph Types**: Each type is optimized for its use case

## Next Steps

- Read the [Architecture Guide](./graph-abstraction-architecture.md)
- Explore the [API Documentation](../api/index.html)
- Check out the [Examples](../examples/)
- Join the community discussions 