# CIM Domain Graph

The graph domain for the Composable Information Machine (CIM). This is the core composition layer that enables other domains to be composed into graphs without creating dependencies.

## Overview

The graph domain provides:
- Graph aggregates (ConceptGraph, DomainGraph)
- Node and edge management
- Graph composition capabilities
- Graph-based workflows
- Conceptual space integration

## Architecture

This domain serves as the composition layer for CIM. Other domains (person, organization, workflow, etc.) can be composed into graphs, but they do not depend on the graph domain. This ensures clean separation of concerns and prevents circular dependencies.

## Features

- **ConceptGraph**: For knowledge representation and conceptual relationships
- **DomainGraph**: For domain model visualization and composition
- **Graph Events**: GraphCreated, NodeAdded, EdgeAdded, etc.
- **Graph Commands**: CreateGraph, AddNode, ConnectNodes, etc.
- **Projections**: GraphSummary, NodeList, etc.

## Usage

```rust
use cim_domain_graph::{ConceptGraph, DomainGraph, GraphCommand};

// Create a new concept graph
let graph = ConceptGraph::new(graph_id, "Knowledge Graph");

// Add nodes from other domains
let person_node = graph.add_node(person_id, NodeType::Person);
let org_node = graph.add_node(org_id, NodeType::Organization);

// Connect nodes
graph.connect_nodes(person_node, org_node, EdgeType::WorksFor);
```

## License

MIT OR Apache-2.0
