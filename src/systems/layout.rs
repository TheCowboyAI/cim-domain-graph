//! Graph layout systems

use crate::{components::*, value_objects::Position3D};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// System that applies force-directed layout to graphs
pub fn force_directed_layout_system(
    graph_query: Query<(&GraphEntity, &GraphLayout)>,
    mut node_query: Query<(&NodeEntity, &mut Position3D)>,
    edge_query: Query<&EdgeEntity>,
) {
    for (graph, layout) in graph_query.iter() {
        if let GraphLayout::ForceDirected {
            spring_strength,
            repulsion_strength,
            damping,
        } = layout
        {
            // Collect nodes for this graph
            let nodes: Vec<(NodeEntity, Position3D)> = node_query
                .iter()
                .filter(|(n, _)| n.graph_id == graph.graph_id)
                .map(|(n, p)| (n.clone(), *p))
                .collect();

            if nodes.is_empty() {
                continue;
            }

            // Calculate forces
            let mut forces: HashMap<NodeId, Position3D> = HashMap::new();

            // Repulsion forces between all nodes
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let (node1, pos1) = &nodes[i];
                    let (node2, pos2) = &nodes[j];

                    let delta = Position3D::new(pos2.x - pos1.x, pos2.y - pos1.y, pos2.z - pos1.z);

                    let distance = delta.magnitude();
                    if distance > 0.001 {
                        let force_magnitude = repulsion_strength / (distance * distance);
                        let force = delta.normalize() * force_magnitude;

                        *forces.entry(node1.node_id).or_default() -= force;
                        *forces.entry(node2.node_id).or_default() += force;
                    }
                }
            }

            // Spring forces for connected nodes
            for edge in edge_query.iter() {
                if edge.graph_id != graph.graph_id {
                    continue;
                }

                let source_pos = nodes
                    .iter()
                    .find(|(n, _)| n.node_id == edge.source)
                    .map(|(_, p)| p);
                let target_pos = nodes
                    .iter()
                    .find(|(n, _)| n.node_id == edge.target)
                    .map(|(_, p)| p);

                if let (Some(source_pos), Some(target_pos)) = (source_pos, target_pos) {
                    let delta = Position3D::new(
                        target_pos.x - source_pos.x,
                        target_pos.y - source_pos.y,
                        target_pos.z - source_pos.z,
                    );

                    let distance = delta.magnitude();
                    if distance > 0.001 {
                        let force_magnitude = spring_strength * (distance - 100.0); // 100.0 is ideal distance
                        let force = delta.normalize() * force_magnitude;

                        *forces.entry(edge.source).or_default() += force;
                        *forces.entry(edge.target).or_default() -= force;
                    }
                }
            }

            // Apply forces with damping
            for (node, mut position) in node_query.iter_mut() {
                if node.graph_id == graph.graph_id {
                    if let Some(force) = forces.get(&node.node_id) {
                        position.x += force.x * (*damping as f64);
                        position.y += force.y * (*damping as f64);
                        position.z += force.z * (*damping as f64);
                    }
                }
            }
        }
    }
}

/// System that applies hierarchical layout to graphs
pub fn hierarchical_layout_system(
    graph_query: Query<(&GraphEntity, &GraphLayout)>,
    mut node_query: Query<(&NodeEntity, &mut Position3D, Option<&NodeType>)>,
    edge_query: Query<&EdgeEntity>,
) {
    for (graph, layout) in graph_query.iter() {
        if let GraphLayout::Hierarchical {
            direction,
            layer_spacing,
            node_spacing,
        } = layout
        {
            // Build adjacency information
            let edges: Vec<_> = edge_query
                .iter()
                .filter(|e| e.graph_id == graph.graph_id)
                .collect();

            // Find root nodes (no incoming edges)
            let mut incoming_edges: HashMap<NodeId, usize> = HashMap::new();
            for edge in &edges {
                *incoming_edges.entry(edge.target).or_insert(0) += 1;
            }

            let root_nodes: Vec<NodeId> = node_query
                .iter()
                .filter(|(n, _, _)| n.graph_id == graph.graph_id)
                .filter(|(n, _, _)| incoming_edges.get(&n.node_id).copied().unwrap_or(0) == 0)
                .map(|(n, _, _)| n.node_id)
                .collect();

            if root_nodes.is_empty() {
                continue;
            }

            // Assign layers using BFS
            let mut node_layers: HashMap<NodeId, usize> = HashMap::new();
            let mut queue = std::collections::VecDeque::new();

            for root in &root_nodes {
                queue.push_back((*root, 0));
            }

            while let Some((node_id, layer)) = queue.pop_front() {
                node_layers.insert(node_id, layer);

                // Find children
                for edge in &edges {
                    if edge.source == node_id && !node_layers.contains_key(&edge.target) {
                        queue.push_back((edge.target, layer + 1));
                    }
                }
            }

            // Count nodes per layer
            let mut layer_counts: HashMap<usize, usize> = HashMap::new();
            let mut layer_indices: HashMap<NodeId, usize> = HashMap::new();

            for (node_id, layer) in &node_layers {
                let index = *layer_counts.entry(*layer).or_insert(0);
                layer_indices.insert(*node_id, index);
                layer_counts.insert(*layer, index + 1);
            }

            // Apply positions
            for (node, mut position, _) in node_query.iter_mut() {
                if node.graph_id == graph.graph_id {
                    if let Some(layer) = node_layers.get(&node.node_id) {
                        let index = layer_indices.get(&node.node_id).copied().unwrap_or(0);
                        let count = layer_counts.get(layer).copied().unwrap_or(1);

                        match direction {
                            LayoutDirection::TopToBottom => {
                                position.x = ((index as f32 - (count - 1) as f32 / 2.0)
                                    * node_spacing)
                                    as f64;
                                position.y = (-(*layer as f32) * layer_spacing) as f64;
                                position.z = 0.0;
                            }
                            LayoutDirection::LeftToRight => {
                                position.x = ((*layer as f32) * layer_spacing) as f64;
                                position.y = ((index as f32 - (count - 1) as f32 / 2.0)
                                    * node_spacing)
                                    as f64;
                                position.z = 0.0;
                            }
                            LayoutDirection::RightToLeft => {
                                position.x = (-(*layer as f32) * layer_spacing) as f64;
                                position.y = ((index as f32 - (count - 1) as f32 / 2.0)
                                    * node_spacing)
                                    as f64;
                                position.z = 0.0;
                            }
                            LayoutDirection::BottomToTop => {
                                position.x = ((index as f32 - (count - 1) as f32 / 2.0)
                                    * node_spacing)
                                    as f64;
                                position.y = ((*layer as f32) * layer_spacing) as f64;
                                position.z = 0.0;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// System that applies circular layout to graphs
pub fn circular_layout_system(
    graph_query: Query<(&GraphEntity, &GraphLayout)>,
    mut node_query: Query<(&NodeEntity, &mut Position3D)>,
) {
    for (graph, layout) in graph_query.iter() {
        if let GraphLayout::Circular { radius } = layout {
            // Collect nodes for this graph
            let nodes: Vec<NodeId> = node_query
                .iter()
                .filter(|(n, _)| n.graph_id == graph.graph_id)
                .map(|(n, _)| n.node_id)
                .collect();

            let node_count = nodes.len();
            if node_count == 0 {
                continue;
            }

            // Calculate positions
            let angle_step = 2.0 * std::f32::consts::PI / node_count as f32;

            for (i, node_id) in nodes.iter().enumerate() {
                let angle = i as f32 * angle_step;
                let x = radius * angle.cos();
                let y = radius * angle.sin();

                // Update position
                for (node, mut position) in node_query.iter_mut() {
                    if node.node_id == *node_id {
                        position.x = x as f64;
                        position.y = y as f64;
                        position.z = 0.0;
                        break;
                    }
                }
            }
        }
    }
}

/// System that applies grid layout to graphs
pub fn grid_layout_system(
    graph_query: Query<(&GraphEntity, &GraphLayout)>,
    mut node_query: Query<(&NodeEntity, &mut Position3D)>,
) {
    for (graph, layout) in graph_query.iter() {
        if let GraphLayout::Grid { columns, spacing } = layout {
            // Collect nodes for this graph
            let nodes: Vec<NodeId> = node_query
                .iter()
                .filter(|(n, _)| n.graph_id == graph.graph_id)
                .map(|(n, _)| n.node_id)
                .collect();

            if nodes.is_empty() {
                continue;
            }

            // Calculate grid positions
            for (i, node_id) in nodes.iter().enumerate() {
                let row = i / *columns;
                let col = i % *columns;

                let x = col as f32 * spacing;
                let y = -(row as f32 * spacing);

                // Update position
                for (node, mut position) in node_query.iter_mut() {
                    if node.node_id == *node_id {
                        position.x = x as f64;
                        position.y = y as f64;
                        position.z = 0.0;
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_circular_layout() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();

        // Create graph with circular layout
        world.spawn((
            GraphEntity {
                graph_id,
                graph_type: GraphType::General,
            },
            GraphLayout::Circular { radius: 100.0 },
        ));

        // Create nodes
        let _node_ids: Vec<_> = (0..4)
            .map(|_| {
                let node_id = NodeId::new();
                world.spawn((NodeEntity { node_id, graph_id }, Position3D::default()));
                node_id
            })
            .collect();

        // Run layout system
        let mut system = IntoSystem::into_system(circular_layout_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify positions are on circle
        let mut query = world.query::<(&NodeEntity, &Position3D)>();
        for (_, position) in query.iter(&world) {
            let distance = (position.x * position.x + position.y * position.y).sqrt();
            assert!((distance - 100.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_grid_layout() {
        let mut world = setup_test_world();
        let graph_id = GraphId::new();

        // Create graph with grid layout
        world.spawn((
            GraphEntity {
                graph_id,
                graph_type: GraphType::General,
            },
            GraphLayout::Grid {
                columns: 2,
                spacing: 50.0,
            },
        ));

        // Create 4 nodes
        for _ in 0..4 {
            world.spawn((
                NodeEntity {
                    node_id: NodeId::new(),
                    graph_id,
                },
                Position3D::default(),
            ));
        }

        // Run layout system
        let mut system = IntoSystem::into_system(grid_layout_system);
        system.initialize(&mut world);
        system.run((), &mut world);
        system.apply_deferred(&mut world);

        // Verify grid positions
        let mut positions: Vec<_> = world
            .query::<&Position3D>()
            .iter(&world)
            .map(|p| (p.x, p.y))
            .collect();
        positions.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap()
                .then(a.0.partial_cmp(&b.0).unwrap())
        });

        assert_eq!(positions[0], (0.0, -50.0)); // Row 1, Col 0
        assert_eq!(positions[1], (50.0, -50.0)); // Row 1, Col 1
        assert_eq!(positions[2], (0.0, 0.0)); // Row 0, Col 0
        assert_eq!(positions[3], (50.0, 0.0)); // Row 0, Col 1
    }
}
