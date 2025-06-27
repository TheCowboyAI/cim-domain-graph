//! Graph Domain Integration Tests

use cim_domain_graph::{
    aggregate::GraphAggregate,
    commands::{AddNode, ConnectEdge, CreateGraph},
    events::{EdgeConnected, GraphCreated, NodeAdded},
    value_objects::{EdgeId, GraphId, NodeId},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph_id = GraphId::new();
        let mut graph = GraphAggregate::new(graph_id.clone());

        assert_eq!(graph.id(), graph_id);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = GraphAggregate::new(GraphId::new());
        let node_id = NodeId::new();

        let result = graph.add_node(node_id.clone());
        assert!(result.is_ok());
        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(&node_id));
    }

    #[test]
    fn test_connect_edge() {
        let mut graph = GraphAggregate::new(GraphId::new());
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        // Add nodes first
        graph.add_node(node1.clone()).unwrap();
        graph.add_node(node2.clone()).unwrap();

        // Connect them
        let edge_id = EdgeId::new();
        let result = graph.connect_edge(edge_id.clone(), node1, node2);

        assert!(result.is_ok());
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_edge(&edge_id));
    }
}
