//! Workflow execution systems for graph-based workflows

use bevy_ecs::prelude::*;
use bevy_ecs::hierarchy::ChildOf;
use crate::components::{NodeEntity, EdgeEntity, GraphEntity, NodeType};
use crate::{NodeId, GraphId};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Component for workflow execution state
#[derive(Component, Debug, Clone)]
pub struct WorkflowExecution {
    pub execution_id: Uuid,
    pub graph_id: GraphId,
    pub current_node: Option<NodeId>,
    pub visited_nodes: HashSet<NodeId>,
    pub execution_path: Vec<NodeId>,
    pub state: WorkflowState,
    pub variables: HashMap<String, serde_json::Value>,
    pub started_at: Instant,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    NotStarted,
    Running,
    Paused,
    Completed,
    Failed(String),
    TimedOut,
}

/// Resource for managing active workflows
#[derive(Resource, Default)]
pub struct WorkflowManager {
    pub active_workflows: HashMap<Uuid, Entity>,
    pub workflow_results: HashMap<Uuid, WorkflowResult>,
}

#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub execution_id: Uuid,
    pub graph_id: GraphId,
    pub final_state: WorkflowState,
    pub execution_path: Vec<NodeId>,
    pub duration: Duration,
    pub output_variables: HashMap<String, serde_json::Value>,
}

/// Event to start a workflow
#[derive(Event, Debug, Clone)]
pub struct StartWorkflowRequest {
    pub graph_id: GraphId,
    pub start_node: Option<NodeId>,
    pub initial_variables: HashMap<String, serde_json::Value>,
    pub timeout: Option<Duration>,
}

/// Event for workflow started
#[derive(Event, Debug, Clone)]
pub struct WorkflowStartedEvent {
    pub execution_id: Uuid,
    pub graph_id: GraphId,
    pub start_node: NodeId,
}

/// Start a workflow execution
pub fn start_workflow_system(
    mut commands: Commands,
    mut start_requests: EventReader<StartWorkflowRequest>,
    mut workflow_manager: ResMut<WorkflowManager>,
    node_query: Query<(&NodeEntity, &NodeType, &ChildOf)>,
    graph_query: Query<&GraphEntity>,
) {
    for request in start_requests.read() {
        // Find start node - either specified or find a node with type Start
        let start_node = if let Some(node_id) = request.start_node {
            Some(node_id)
        } else {
            // Find a start node in the graph
            node_query
                .iter()
                .find(|(_node, node_type, child_of)| {
                    if let Ok(graph) = graph_query.get(child_of.parent()) {
                        graph.graph_id == request.graph_id && **node_type == NodeType::Start
                    } else {
                        false
                    }
                })
                .map(|(node, _, _)| node.node_id)
        };
        
        if let Some(start_node) = start_node {
            let execution_id = Uuid::new_v4();
            
            // Create workflow execution entity
            let workflow_entity = commands.spawn(WorkflowExecution {
                execution_id,
                graph_id: request.graph_id,
                current_node: Some(start_node),
                visited_nodes: HashSet::from([start_node]),
                execution_path: vec![start_node],
                state: WorkflowState::Running,
                variables: request.initial_variables.clone(),
                started_at: Instant::now(),
                timeout: request.timeout,
            }).id();
            
            // Register in manager
            workflow_manager.active_workflows.insert(execution_id, workflow_entity);
            
            // Trigger started event
            let graph_id = request.graph_id;
            commands.queue(move |world: &mut World| {
                world.send_event(WorkflowStartedEvent {
                    execution_id,
                    graph_id,
                    start_node,
                });
            });
        }
    }
}

/// Event to advance workflow
#[derive(Event, Debug, Clone)]
pub struct AdvanceWorkflowRequest {
    pub execution_id: Uuid,
    pub next_node: Option<NodeId>,
}

/// Event for workflow advanced
#[derive(Event, Debug, Clone)]
pub struct WorkflowAdvancedEvent {
    pub execution_id: Uuid,
    pub from_node: NodeId,
    pub to_node: NodeId,
}

/// Advance workflow to next step
pub fn advance_workflow_system(
    mut commands: Commands,
    mut advance_requests: EventReader<AdvanceWorkflowRequest>,
    mut workflow_query: Query<&mut WorkflowExecution>,
    workflow_manager: Res<WorkflowManager>,
    edge_query: Query<(&EdgeEntity, &ChildOf)>,
    graph_query: Query<&GraphEntity>,
) {
    for request in advance_requests.read() {
        if let Some(&entity) = workflow_manager.active_workflows.get(&request.execution_id) {
            if let Ok(mut workflow) = workflow_query.get_mut(entity) {
                if workflow.state != WorkflowState::Running {
                    continue;
                }
                
                if let Some(current_node) = workflow.current_node {
                    // Determine next node
                    let next_node = if let Some(specified_node) = request.next_node {
                        Some(specified_node)
                    } else {
                        // Find outgoing edges from current node
                        let mut candidates = Vec::new();
                        
                        for (edge, child_of) in &edge_query {
                            if let Ok(graph) = graph_query.get(child_of.parent()) {
                                if graph.graph_id == workflow.graph_id && edge.source == current_node {
                                    candidates.push(edge.target);
                                }
                            }
                        }
                        
                        // For now, take the first candidate
                        candidates.first().cloned()
                    };
                    
                    if let Some(next_node) = next_node {
                        // Update workflow state
                        workflow.current_node = Some(next_node);
                        workflow.visited_nodes.insert(next_node);
                        workflow.execution_path.push(next_node);
                        
                        // Trigger advanced event
                        let event = WorkflowAdvancedEvent {
                            execution_id: request.execution_id,
                            from_node: current_node,
                            to_node: next_node,
                        };
                        commands.queue(move |world: &mut World| {
                            world.send_event(event);
                        });
                    }
                }
            }
        }
    }
}

/// Event to complete workflow
#[derive(Event, Debug, Clone)]
pub struct CompleteWorkflowRequest {
    pub execution_id: Uuid,
    pub output_variables: HashMap<String, serde_json::Value>,
}

/// Event for workflow completed
#[derive(Event, Debug, Clone)]
pub struct WorkflowCompletedEvent {
    pub execution_id: Uuid,
    pub result: WorkflowResult,
}

/// Complete a workflow
pub fn complete_workflow_system(
    mut commands: Commands,
    mut complete_requests: EventReader<CompleteWorkflowRequest>,
    mut workflow_query: Query<&mut WorkflowExecution>,
    mut workflow_manager: ResMut<WorkflowManager>,
) {
    for request in complete_requests.read() {
        if let Some(&entity) = workflow_manager.active_workflows.get(&request.execution_id) {
            if let Ok(mut workflow) = workflow_query.get_mut(entity) {
                if workflow.state == WorkflowState::Running {
                    workflow.state = WorkflowState::Completed;
                    
                    let result = WorkflowResult {
                        execution_id: workflow.execution_id,
                        graph_id: workflow.graph_id,
                        final_state: WorkflowState::Completed,
                        execution_path: workflow.execution_path.clone(),
                        duration: workflow.started_at.elapsed(),
                        output_variables: request.output_variables.clone(),
                    };
                    
                    // Store result
                    workflow_manager.workflow_results.insert(workflow.execution_id, result.clone());
                    
                    // Remove from active workflows
                    workflow_manager.active_workflows.remove(&workflow.execution_id);
                    
                    // Trigger completed event
                    let event = WorkflowCompletedEvent {
                        execution_id: workflow.execution_id,
                        result,
                    };
                    commands.queue(move |world: &mut World| {
                        world.send_event(event);
                    });
                    
                    // Despawn entity
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Handle workflow timeouts
pub fn timeout_workflows_system(
    mut commands: Commands,
    mut workflow_query: Query<(Entity, &mut WorkflowExecution)>,
    mut workflow_manager: ResMut<WorkflowManager>,
) {
    let now = Instant::now();
    
    for (entity, mut workflow) in &mut workflow_query {
        if workflow.state == WorkflowState::Running {
            if let Some(timeout) = workflow.timeout {
                if workflow.started_at + timeout < now {
                    workflow.state = WorkflowState::TimedOut;
                    
                    let result = WorkflowResult {
                        execution_id: workflow.execution_id,
                        graph_id: workflow.graph_id,
                        final_state: WorkflowState::TimedOut,
                        execution_path: workflow.execution_path.clone(),
                        duration: workflow.started_at.elapsed(),
                        output_variables: workflow.variables.clone(),
                    };
                    
                    // Store result
                    workflow_manager.workflow_results.insert(workflow.execution_id, result.clone());
                    
                    // Remove from active workflows
                    workflow_manager.active_workflows.remove(&workflow.execution_id);
                    
                    // Trigger timeout event
                    let event = WorkflowTimeoutEvent {
                        execution_id: workflow.execution_id,
                        duration: workflow.started_at.elapsed(),
                    };
                    commands.queue(move |world: &mut World| {
                        world.send_event(event);
                    });
                    
                    // Despawn entity
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Event for workflow timeout
#[derive(Event, Debug, Clone)]
pub struct WorkflowTimeoutEvent {
    pub execution_id: Uuid,
    pub duration: Duration,
}

/// Event to pause workflow
#[derive(Event, Debug, Clone)]
pub struct PauseWorkflowRequest {
    pub execution_id: Uuid,
}

/// Event to resume workflow
#[derive(Event, Debug, Clone)]
pub struct ResumeWorkflowRequest {
    pub execution_id: Uuid,
}

/// System to pause/resume workflows
pub fn pause_resume_workflow_system(
    mut pause_requests: EventReader<PauseWorkflowRequest>,
    mut resume_requests: EventReader<ResumeWorkflowRequest>,
    mut workflow_query: Query<&mut WorkflowExecution>,
    workflow_manager: Res<WorkflowManager>,
) {
    // Handle pause requests
    for request in pause_requests.read() {
        if let Some(&entity) = workflow_manager.active_workflows.get(&request.execution_id) {
            if let Ok(mut workflow) = workflow_query.get_mut(entity) {
                if workflow.state == WorkflowState::Running {
                    workflow.state = WorkflowState::Paused;
                }
            }
        }
    }
    
    // Handle resume requests
    for request in resume_requests.read() {
        if let Some(&entity) = workflow_manager.active_workflows.get(&request.execution_id) {
            if let Ok(mut workflow) = workflow_query.get_mut(entity) {
                if workflow.state == WorkflowState::Paused {
                    workflow.state = WorkflowState::Running;
                }
            }
        }
    }
}

/// Plugin to register workflow systems
pub struct WorkflowSystemsPlugin;

impl bevy_app::Plugin for WorkflowSystemsPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            .init_resource::<WorkflowManager>()
            .add_event::<StartWorkflowRequest>()
            .add_event::<WorkflowStartedEvent>()
            .add_event::<AdvanceWorkflowRequest>()
            .add_event::<WorkflowAdvancedEvent>()
            .add_event::<CompleteWorkflowRequest>()
            .add_event::<WorkflowCompletedEvent>()
            .add_event::<WorkflowTimeoutEvent>()
            .add_event::<PauseWorkflowRequest>()
            .add_event::<ResumeWorkflowRequest>()
            .add_systems(
                bevy_app::Update,
                (
                    start_workflow_system,
                    advance_workflow_system,
                    complete_workflow_system,
                    timeout_workflows_system,
                    pause_resume_workflow_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_state() {
        let state = WorkflowState::Running;
        assert_eq!(state, WorkflowState::Running);
        
        let failed = WorkflowState::Failed("Test error".to_string());
        assert!(matches!(failed, WorkflowState::Failed(_)));
    }
    
    #[test]
    fn test_workflow_execution_creation() {
        let execution = WorkflowExecution {
            execution_id: Uuid::new_v4(),
            graph_id: GraphId::new(),
            current_node: None,
            visited_nodes: HashSet::new(),
            execution_path: Vec::new(),
            state: WorkflowState::NotStarted,
            variables: HashMap::new(),
            started_at: Instant::now(),
            timeout: Some(Duration::from_secs(60)),
        };
        
        assert_eq!(execution.state, WorkflowState::NotStarted);
        assert!(execution.visited_nodes.is_empty());
    }
}