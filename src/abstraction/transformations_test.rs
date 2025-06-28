//! Integration tests for graph transformations

#[cfg(test)]
mod tests {
    use crate::{
        GraphId, NodeId, EdgeId,
        abstraction::{
            GraphType, GraphImplementation, NodeData, EdgeData, Position3D,
            DefaultGraphTransformer, GraphTransformer, TransformationOptions,
        },
    };
    use std::collections::HashMap;
    use serde_json::json;

    /// Test transformation from context to workflow graph
    #[test]
    fn test_context_to_workflow_transformation() {
        let transformer = DefaultGraphTransformer::new();
        let graph_id = GraphId::new();
        
        // Create a context graph
        let mut context_graph = GraphType::new_context(graph_id, "Test Context");
        
        // Add nodes
        let node1 = NodeId::new();
        context_graph.add_node(node1, NodeData {
            node_type: "user".to_string(),
            position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("name".to_string(), json!("Alice"));
                meta.insert("role".to_string(), json!("admin"));
                meta
            },
        }).unwrap();
        
        let node2 = NodeId::new();
        context_graph.add_node(node2, NodeData {
            node_type: "resource".to_string(),
            position: Position3D { x: 10.0, y: 0.0, z: 0.0 },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("name".to_string(), json!("Database"));
                meta.insert("type".to_string(), json!("system"));
                meta
            },
        }).unwrap();
        
        // Add edge
        let edge = EdgeId::new();
        context_graph.add_edge(edge, node1, node2, EdgeData {
            edge_type: "accesses".to_string(),
            metadata: HashMap::new(),
        }).unwrap();
        
        // Transform to workflow
        let options = TransformationOptions {
            preserve_metadata: true,
            allow_data_loss: true,
            ..Default::default()
        };
        
        let workflow_graph = transformer.transform(&context_graph, "workflow", options).unwrap();
        
        // Verify transformation
        let nodes = workflow_graph.list_nodes();
        assert_eq!(nodes.len(), 2);
        
        // Check that workflow-specific metadata was added
        for (_, node_data) in &nodes {
            assert!(node_data.metadata.contains_key("step_type"));
        }
        
        // Check that original metadata is preserved
        let node1_data = workflow_graph.get_node(node1).unwrap();
        assert_eq!(node1_data.metadata.get("name").unwrap(), &json!("Alice"));
        
        // Check edges
        let edges = workflow_graph.list_edges();
        assert_eq!(edges.len(), 1);
        
        // Check that edge has workflow metadata
        let (edge_data, _, _) = workflow_graph.get_edge(edge).unwrap();
        assert!(edge_data.metadata.contains_key("flow_type"));
    }
    
    /// Test transformation with custom mappings
    #[test]
    fn test_transformation_with_custom_mappings() {
        let transformer = DefaultGraphTransformer::new();
        let graph_id = GraphId::new();
        
        // Create a workflow graph
        let mut workflow_graph = GraphType::new_workflow(graph_id, "Test Workflow");
        
        // Add nodes with specific types
        let node1 = NodeId::new();
        workflow_graph.add_node(node1, NodeData {
            node_type: "start".to_string(),
            position: Position3D::default(),
            metadata: HashMap::new(),
        }).unwrap();
        
        let node2 = NodeId::new();
        workflow_graph.add_node(node2, NodeData {
            node_type: "process".to_string(),
            position: Position3D::default(),
            metadata: HashMap::new(),
        }).unwrap();
        
        // Add edge
        let edge = EdgeId::new();
        workflow_graph.add_edge(edge, node1, node2, EdgeData {
            edge_type: "sequence".to_string(),
            metadata: HashMap::new(),
        }).unwrap();
        
        // Create custom mappings
        let mut options = TransformationOptions {
            preserve_metadata: true,
            allow_data_loss: true,
            ..Default::default()
        };
        
        options.node_type_mappings.insert("start".to_string(), "entry".to_string());
        options.node_type_mappings.insert("process".to_string(), "action".to_string());
        options.edge_type_mappings.insert("sequence".to_string(), "flow".to_string());
        
        // Transform to context
        let context_graph = transformer.transform(&workflow_graph, "context", options).unwrap();
        
        // Verify mappings were applied
        let node1_data = context_graph.get_node(node1).unwrap();
        assert_eq!(node1_data.node_type, "entry");
        
        let node2_data = context_graph.get_node(node2).unwrap();
        assert_eq!(node2_data.node_type, "action");
        
        let (edge_data, _, _) = context_graph.get_edge(edge).unwrap();
        assert_eq!(edge_data.edge_type, "flow");
    }
    
    /// Test data loss prevention
    #[test]
    fn test_data_loss_prevention() {
        let transformer = DefaultGraphTransformer::new();
        let graph_id = GraphId::new();
        
        // Create a workflow graph with execution-specific metadata
        let mut workflow_graph = GraphType::new_workflow(graph_id, "Test Workflow");
        
        let node = NodeId::new();
        workflow_graph.add_node(node, NodeData {
            node_type: "process".to_string(),
            position: Position3D::default(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("execution_time".to_string(), json!(30));
                meta.insert("retry_count".to_string(), json!(3));
                meta
            },
        }).unwrap();
        
        // Try to transform without allowing data loss
        let options = TransformationOptions {
            preserve_metadata: true,
            allow_data_loss: false,
            ..Default::default()
        };
        
        // This should fail due to data loss
        let result = transformer.transform(&workflow_graph, "concept", options);
        assert!(result.is_err());
        
        // Check that preview correctly identifies data loss
        let warnings = transformer.preview_data_loss(&workflow_graph, "concept");
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("execution time"));
    }
    
    /// Test round-trip transformation
    #[test]
    fn test_round_trip_transformation() {
        let transformer = DefaultGraphTransformer::new();
        let graph_id = GraphId::new();
        
        // Create original context graph
        let mut original = GraphType::new_context(graph_id, "Original");
        
        let node = NodeId::new();
        original.add_node(node, NodeData {
            node_type: "entity".to_string(),
            position: Position3D { x: 1.0, y: 2.0, z: 3.0 },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("custom_field".to_string(), json!("value"));
                meta
            },
        }).unwrap();
        
        let options = TransformationOptions {
            preserve_metadata: true,
            allow_data_loss: true,
            ..Default::default()
        };
        
        // Transform context -> workflow -> context
        let workflow = transformer.transform(&original, "workflow", options.clone()).unwrap();
        let round_trip = transformer.transform(&workflow, "context", options).unwrap();
        
        // Verify core data is preserved
        let original_node = original.get_node(node).unwrap();
        let round_trip_node = round_trip.get_node(node).unwrap();
        
        assert_eq!(original_node.node_type, round_trip_node.node_type);
        assert_eq!(original_node.position, round_trip_node.position);
        
        // Custom field should still exist
        assert_eq!(
            round_trip_node.metadata.get("custom_field"),
            Some(&json!("value"))
        );
    }
    
    /// Test all supported transformation paths
    #[test]
    fn test_all_transformation_paths() {
        let transformer = DefaultGraphTransformer::new();
        
        let types = vec!["context", "concept", "workflow", "ipld"];
        
        for from_type in &types {
            for to_type in &types {
                if from_type != to_type {
                    assert!(
                        transformer.is_transformation_supported(from_type, to_type),
                        "Transformation from {} to {} should be supported",
                        from_type, to_type
                    );
                }
            }
        }
    }
    
    /// Test metadata preservation
    #[test]
    fn test_metadata_preservation() {
        let transformer = DefaultGraphTransformer::new();
        let graph_id = GraphId::new();
        
        // Create a graph with rich metadata
        let mut source = GraphType::new_concept(graph_id, "Source Graph");
        
        let node = NodeId::new();
        source.add_node(node, NodeData {
            node_type: "concept".to_string(),
            position: Position3D::default(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("field1".to_string(), json!("value1"));
                meta.insert("field2".to_string(), json!(42));
                meta.insert("field3".to_string(), json!({"nested": "object"}));
                meta
            },
        }).unwrap();
        
        // Transform with metadata preservation
        let options = TransformationOptions {
            preserve_metadata: true,
            allow_data_loss: true,
            ..Default::default()
        };
        
        let target = transformer.transform(&source, "workflow", options).unwrap();
        
        // Check that all original metadata is preserved
        let target_node = target.get_node(node).unwrap();
        assert_eq!(target_node.metadata.get("field1"), Some(&json!("value1")));
        assert_eq!(target_node.metadata.get("field2"), Some(&json!(42)));
        assert_eq!(target_node.metadata.get("field3"), Some(&json!({"nested": "object"})));
        
        // Check that transformation metadata was added
        let graph_metadata = target.get_metadata();
        assert!(graph_metadata.properties.contains_key("transformed_from"));
    }
} 