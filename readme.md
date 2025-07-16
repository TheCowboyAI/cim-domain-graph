# CIM Graph Domain

The Graph Domain is the core visualization and interaction layer for the Composable Information Machine (CIM). It provides event-driven graph operations, multiple graph types, and a powerful abstraction layer for transformations and composition.

## Features

- ✅ **Event-Driven Architecture**: All operations through domain events
- ✅ **CQRS Implementation**: Separated command and query models
- ✅ **Multiple Graph Types**: Workflow, Concept, Context, and IPLD graphs
- ✅ **Graph Abstraction Layer**: Unified interface for all graph types
- ✅ **Transformations**: Convert between graph types with metadata preservation
- ✅ **Composition**: Combine multiple graphs with conflict resolution
- ✅ **Bevy ECS Integration**: Real-time visualization support

## Quick Start

### Basic Graph Operations

```rust
use cim_domain_graph::{
    GraphId, NodeId, EdgeId,
    commands::{GraphCommand, NodeCommand, EdgeCommand},
    handlers::UnifiedGraphCommandHandler,
};

// Create a graph
let command = GraphCommand::CreateGraph {
    name: "My Workflow".to_string(),
    description: "Order processing workflow".to_string(),
    metadata: HashMap::new(),
};

let events = handler.handle_graph_command(command).await?;

// Add nodes
let node_command = NodeCommand::Add {
    graph_id,
    node_type: "Process".to_string(),
    metadata: hashmap!{
        "name" => json!("Receive Order"),
        "duration" => json!(30),
    },
};

handler.handle_node_command(node_command).await?;
```

### Using the Abstraction Layer

```rust
use cim_domain_graph::abstraction::{
    GraphType, GraphImplementation,
    DefaultGraphTransformer, GraphTransformer,
    DefaultGraphComposer, GraphComposer,
};

// Create different graph types
let workflow = GraphType::new_workflow(GraphId::new(), "Order Process");
let concept = GraphType::new_concept(GraphId::new(), "Business Concepts");

// Transform between types
let transformer = DefaultGraphTransformer::new();
let concept_from_workflow = transformer.transform(
    &workflow,
    "concept",
    TransformationOptions::default()
)?;

// Compose multiple graphs
let composer = DefaultGraphComposer::new();
let unified = composer.compose(
    &[&workflow, &concept],
    "context",
    CompositionOptions::default()
)?;
```

### Bevy ECS Integration

```rust
use bevy::prelude::*;
use cim_domain_graph::{
    abstraction::GraphAbstractionPlugin,
    components::{GraphEntity, NodeEntity, EdgeEntity},
    events::{GraphCreated, NodeAdded, EdgeAdded},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphAbstractionPlugin {
            command_handler: Arc::new(command_handler),
            query_handler: Arc::new(query_handler),
        })
        .add_systems(Update, visualize_graphs)
        .run();
}

fn visualize_graphs(
    graphs: Query<&GraphEntity>,
    nodes: Query<(&NodeEntity, &Transform)>,
) {
    // Visualization logic
}
```

## Graph Types

### 1. Workflow Graphs
Represent business processes with sequential and parallel flows.

```rust
let workflow = GraphType::new_workflow(id, "Order Processing");
workflow.add_node(NodeData {
    id: NodeId::new(),
    node_type: "Process".to_string(),
    position: Some(Position3D { x: 0.0, y: 0.0, z: 0.0 }),
    metadata: hashmap!{ "step" => json!("validate") },
})?;
```

### 2. Concept Graphs
Represent knowledge and semantic relationships.

```rust
let concepts = GraphType::new_concept(id, "Domain Knowledge");
concepts.add_node(NodeData {
    id: NodeId::new(),
    node_type: "Concept".to_string(),
    metadata: hashmap!{ 
        "category" => json!("business"),
        "embedding" => json!([0.1, 0.2, 0.3]),
    },
})?;
```

### 3. Context Graphs
General-purpose graphs for various contexts.

```rust
let context = GraphType::new_context(id, "System Overview");
// Supports any node/edge structure
```

### 4. IPLD Graphs
Content-addressed graphs with CID references.

```rust
let ipld = GraphType::new_ipld(id, "Distributed Data");
// Nodes reference content by CID
```

## Transformations

Transform graphs between types while preserving metadata:

```rust
let transformer = DefaultGraphTransformer::new();

// Configure transformation
let mut options = TransformationOptions::default();
options.node_type_mapping.insert(
    "Process".to_string(),
    "Activity".to_string(),
);

// Transform with preview
let preview = transformer.preview_data_loss(&source, "workflow")?;
if preview.is_empty() {
    let target = transformer.transform(&source, "workflow", options)?;
}
```

## Composition

Combine multiple graphs with conflict resolution:

```rust
let composer = DefaultGraphComposer::new();

// Configure composition
let mut options = CompositionOptions::default();
options.conflict_resolution = ConflictResolution::Merge;
options.node_id_prefix = Some("merged".to_string());

// Preview conflicts
let conflicts = composer.preview_conflicts(&[&graph1, &graph2])?;

// Compose graphs
let composed = composer.compose(
    &[&graph1, &graph2],
    "context",
    options
)?;
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Presentation Layer                      │
│                    (Bevy ECS)                            │
├─────────────────────────────────────────────────────────┤
│                  Abstraction Layer                        │
│         (Transformations & Composition)                   │
├─────────────────────────────────────────────────────────┤
│                  Application Layer                        │
│            (Command & Query Handlers)                     │
├─────────────────────────────────────────────────────────┤
│                    Domain Layer                           │
│              (Aggregates & Events)                        │
├─────────────────────────────────────────────────────────┤
│                Infrastructure Layer                       │
│              (NATS & Event Store)                        │
└─────────────────────────────────────────────────────────┘
```

## Testing

Run all tests:
```bash
cargo test
```

Run specific test categories:
```bash
# Abstraction layer tests
cargo test abstraction

# Transformation tests
cargo test transformation

# Composition tests
cargo test composition
```

## Examples

See the `examples/` directory for complete examples:

- `graph_abstraction_demo.rs` - Basic abstraction usage
- `graph_transformation_demo.rs` - Transformation examples
- `graph_composition_demo.rs` - Composition examples
- `graph_abstraction_integration_demo.rs` - Full Bevy integration (needs fixes)

## Performance

The abstraction layer is designed for performance:

- Minimal copying during transformations
- Efficient HashMap-based lookups
- Lazy evaluation where possible
- Support for large graphs (tested with 10K+ nodes)

## Contributing

1. All changes must maintain event-driven architecture
2. No CRUD operations - everything through events
3. Maintain test coverage above 90%
4. Update documentation for API changes

## License

Part of the CIM project. See root LICENSE file.
