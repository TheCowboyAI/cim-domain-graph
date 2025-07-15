//! Graph domain ECS systems

pub mod lifecycle;
pub mod node_management;
pub mod edge_management;
pub mod layout;
pub mod spatial;
pub mod workflow;
pub mod query;
pub mod advanced_layout_system;

// Re-export all systems
pub use lifecycle::{
    create_graph_system,
    update_graph_system,
    archive_graph_system,
};

pub use node_management::{
    add_node_system,
    update_node_system,
    remove_node_system,
};

pub use edge_management::{
    add_edge_system,
    update_edge_system,
    remove_edge_system,
    validate_edges_system,
};

pub use layout::{
    force_directed_layout_system,
    hierarchical_layout_system,
    circular_layout_system,
    grid_layout_system,
};

pub use spatial::*;
pub use workflow::*;
pub use query::*;
pub use advanced_layout_system::{
    AdvancedLayoutType, AdvancedLayoutConfig, ApplyAdvancedLayout,
    AdvancedLayoutPlugin, LayoutQualityMetrics,
    apply_advanced_layout_system, calculate_layout_quality_system,
}; 