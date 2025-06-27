//! ECS Systems for the Graph domain
//!
//! Systems implement the behavior that operates on components.
//! They process commands, emit events, and update component state.

pub mod lifecycle;
pub mod node_management;
pub mod edge_management;
pub mod layout;
pub mod spatial;
pub mod workflow;
pub mod queries;

// Re-export commonly used systems
pub use lifecycle::{
    create_graph_system, update_graph_system, archive_graph_system,
    process_graph_events_system,
};

pub use node_management::{
    add_node_system, update_node_system, remove_node_system,
    process_node_events_system,
};

pub use edge_management::{
    connect_nodes_system, update_edge_system, disconnect_nodes_system,
    process_edge_events_system,
};

pub use layout::{
    apply_layout_system, force_directed_layout_system,
    hierarchical_layout_system, update_positions_system,
};

pub use spatial::{
    update_spatial_index_system, find_nodes_in_region_system,
    cluster_nearby_nodes_system,
};

pub use workflow::{
    start_workflow_system, advance_workflow_system,
    complete_workflow_system, timeout_workflows_system,
};

pub use queries::{
    find_nodes_by_type_system, find_connected_nodes_system,
    calculate_shortest_path_system,
}; 