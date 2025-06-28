//! Abstract graph query handler that works with any graph implementation

use crate::{
    aggregate::abstract_graph::AbstractGraph,
    queries::{GraphInfo, GraphQueryError, GraphQueryResult, NodeInfo},
    GraphId, NodeId,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Repository trait for querying abstract graphs
#[async_trait]
pub trait AbstractGraphQueryRepository: Send + Sync {
    /// Load the full abstract graph
    async fn load_abstract_graph(&self, graph_id: GraphId) -> GraphQueryResult<AbstractGraph>;

    /// Get graph type for a graph
    async fn get_graph_type(&self, graph_id: GraphId) -> GraphQueryResult<String>;

    /// Get all graph IDs
    async fn list_graph_ids(&self) -> GraphQueryResult<Vec<GraphId>>;
}

/// Query handler that works with abstract graphs
pub struct AbstractGraphQueryHandler {
    repository: Arc<dyn AbstractGraphQueryRepository>,
}

impl AbstractGraphQueryHandler {
    /// Create a new abstract graph query handler
    pub fn new(repository: Arc<dyn AbstractGraphQueryRepository>) -> Self {
        Self { repository }
    }

    /// Get graph info from abstract graph
    pub async fn get_graph_info(&self, graph_id: GraphId) -> GraphQueryResult<GraphInfo> {
        let graph = self.repository.load_abstract_graph(graph_id).await?;

        Ok(GraphInfo {
            graph_id: graph.id(),
            name: graph.name(),
            description: String::new(), // AbstractGraph doesn't store description
            node_count: graph.node_count(),
            edge_count: graph.edge_count(),
            created_at: chrono::Utc::now(), // Would come from event store
            last_modified: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Get all nodes in a graph
    pub async fn get_nodes_in_graph(&self, graph_id: GraphId) -> GraphQueryResult<Vec<NodeInfo>> {
        let graph = self.repository.load_abstract_graph(graph_id).await?;

        let nodes: Vec<NodeInfo> = graph
            .list_nodes()
            .into_iter()
            .map(|(node_id, node_data)| NodeInfo {
                node_id,
                graph_id,
                node_type: node_data.node_type,
                position_2d: None,
                position_3d: Some(crate::value_objects::Position3D::new(
                    node_data.position.x,
                    node_data.position.y,
                    node_data.position.z,
                )),
                metadata: node_data.metadata,
            })
            .collect();

        Ok(nodes)
    }

    /// Find shortest path between two nodes
    pub async fn find_shortest_path(
        &self,
        graph_id: GraphId,
        source: NodeId,
        target: NodeId,
    ) -> GraphQueryResult<Option<Vec<NodeId>>> {
        let graph = self.repository.load_abstract_graph(graph_id).await?;

        // Build adjacency list
        let mut adjacency: std::collections::HashMap<NodeId, Vec<NodeId>> =
            std::collections::HashMap::new();

        for (_, _, src, tgt) in graph.list_edges() {
            adjacency.entry(src).or_default().push(tgt);
            // For undirected graphs, add reverse edge
            adjacency.entry(tgt).or_default().push(src);
        }

        // Use BFS for unweighted shortest path
        use std::collections::{HashSet, VecDeque};

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: std::collections::HashMap<NodeId, NodeId> =
            std::collections::HashMap::new();

        queue.push_back(source);
        visited.insert(source);

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = target;

                while node != source {
                    path.push(node);
                    node = *parent.get(&node).ok_or_else(|| {
                        GraphQueryError::InvalidQuery("Path reconstruction failed".to_string())
                    })?;
                }
                path.push(source);
                path.reverse();

                return Ok(Some(path));
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find connected components in the graph
    pub async fn find_connected_components(
        &self,
        graph_id: GraphId,
    ) -> GraphQueryResult<Vec<Vec<NodeId>>> {
        let graph = self.repository.load_abstract_graph(graph_id).await?;
        let nodes: Vec<NodeId> = graph.list_nodes().into_iter().map(|(id, _)| id).collect();

        // Build adjacency list
        let mut adjacency: std::collections::HashMap<NodeId, Vec<NodeId>> =
            std::collections::HashMap::new();

        for node_id in &nodes {
            adjacency.insert(*node_id, Vec::new());
        }

        for (_, _, src, tgt) in graph.list_edges() {
            adjacency.get_mut(&src).unwrap().push(tgt);
            adjacency.get_mut(&tgt).unwrap().push(src);
        }

        // DFS to find components
        let mut visited = std::collections::HashSet::new();
        let mut components = Vec::new();

        for &node in &nodes {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                let mut stack = vec![node];

                while let Some(current) = stack.pop() {
                    if visited.insert(current) {
                        component.push(current);

                        if let Some(neighbors) = adjacency.get(&current) {
                            for &neighbor in neighbors {
                                if !visited.contains(&neighbor) {
                                    stack.push(neighbor);
                                }
                            }
                        }
                    }
                }

                if !component.is_empty() {
                    components.push(component);
                }
            }
        }

        Ok(components)
    }
}

/// Graph statistics structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_degree: f64,
    pub max_degree: usize,
    pub min_degree: usize,
    pub node_type_distribution: std::collections::HashMap<String, usize>,
    pub density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        abstraction::{NodeData, EdgeData, GraphType, Position3D},
        GraphId, NodeId,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    

    // Mock implementation for testing
    struct MockQueryRepository {
        graphs: std::sync::Mutex<HashMap<GraphId, (AbstractGraph, String)>>,
    }

    impl MockQueryRepository {
        fn new() -> Self {
            Self {
                graphs: std::sync::Mutex::new(HashMap::new()),
            }
        }

        fn add_graph(&self, graph: AbstractGraph, graph_type: String) {
            let mut graphs = self.graphs.lock().unwrap();
            graphs.insert(graph.id(), (graph, graph_type));
        }
    }

    #[async_trait]
    impl AbstractGraphQueryRepository for MockQueryRepository {
        async fn load_abstract_graph(&self, graph_id: GraphId) -> GraphQueryResult<AbstractGraph> {
            let graphs = self.graphs.lock().unwrap();
            graphs
                .get(&graph_id)
                .map(|(g, _)| g.clone())
                .ok_or(GraphQueryError::GraphNotFound(graph_id))
        }

        async fn get_graph_type(&self, graph_id: GraphId) -> GraphQueryResult<String> {
            let graphs = self.graphs.lock().unwrap();
            graphs
                .get(&graph_id)
                .map(|(_, t)| t.clone())
                .ok_or(GraphQueryError::GraphNotFound(graph_id))
        }

        async fn list_graph_ids(&self) -> GraphQueryResult<Vec<GraphId>> {
            let graphs = self.graphs.lock().unwrap();
            Ok(graphs.keys().cloned().collect())
        }
    }

    #[tokio::test]
    async fn test_abstract_query_handler() {
        let repository = Arc::new(MockQueryRepository::new());
        let handler = AbstractGraphQueryHandler::new(repository.clone());

        // Create a test graph
        let graph_id = GraphId::new();
        let mut graph = AbstractGraph::new(GraphType::new_context(graph_id, "Test Graph"));

        // Add some nodes
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        graph
            .add_node(
                node1,
                NodeData {
                    node_type: "start".to_string(),
                    position: Position3D::default(),
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        graph
            .add_node(
                node2,
                NodeData {
                    node_type: "process".to_string(),
                    position: Position3D::default(),
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        graph
            .add_node(
                node3,
                NodeData {
                    node_type: "end".to_string(),
                    position: Position3D::default(),
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        // Add edges
        graph
            .add_edge(
                crate::EdgeId::new(),
                node1,
                node2,
                EdgeData {
                    edge_type: "sequence".to_string(),
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        graph
            .add_edge(
                crate::EdgeId::new(),
                node2,
                node3,
                EdgeData {
                    edge_type: "sequence".to_string(),
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        repository.add_graph(graph, "context".to_string());

        // Test get graph info
        let graph_info = handler.get_graph_info(graph_id).await.unwrap();
        assert_eq!(graph_info.graph_id, graph_id);
        assert_eq!(graph_info.name, "Test Graph");
        assert_eq!(graph_info.node_count, 3);
        assert_eq!(graph_info.edge_count, 2);

        // Test get nodes
        let nodes = handler.get_nodes_in_graph(graph_id).await.unwrap();
        assert_eq!(nodes.len(), 3);

        // Test shortest path
        let path = handler
            .find_shortest_path(graph_id, node1, node3)
            .await
            .unwrap();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], node1);
        assert_eq!(path[1], node2);
        assert_eq!(path[2], node3);

        // Test connected components
        let components = handler.find_connected_components(graph_id).await.unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 3);
    }
}
