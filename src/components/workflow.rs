//! Workflow-related ECS components

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::NodeId;

/// Workflow state for a graph
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub status: WorkflowStatus,
    pub current_step: Option<NodeId>,
    pub execution_path: Vec<NodeId>,
    pub context: HashMap<String, serde_json::Value>,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            status: WorkflowStatus::NotStarted,
            current_step: None,
            execution_path: Vec::new(),
            context: HashMap::new(),
        }
    }
}

/// Workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow has not started
    NotStarted,
    /// Workflow is running
    Running,
    /// Workflow is paused
    Paused,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
}

/// Workflow step configuration
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_id: String,
    pub step_type: StepType,
    pub timeout: Option<std::time::Duration>,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepType {
    /// Manual task requiring user input
    Manual,
    /// Automated task
    Automated,
    /// Decision point
    Decision,
    /// Parallel execution
    Parallel,
    /// Sub-workflow
    SubWorkflow { workflow_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed { delay_ms: u64 },
    /// Exponential backoff
    Exponential { initial_ms: u64, factor: f32 },
    /// No retry
    None,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::Exponential {
                initial_ms: 1000,
                factor: 2.0,
            },
        }
    }
}

/// Workflow transition between steps
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub from_step: NodeId,
    pub to_step: NodeId,
    pub condition: TransitionCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    /// Always transition
    Always,
    /// Transition if expression evaluates to true
    Expression { expr: String },
    /// Transition on specific event
    Event { event_type: String },
    /// Manual approval required
    Approval { approver_role: String },
}

/// Workflow metadata
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl Default for WorkflowMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            name: String::new(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
} 