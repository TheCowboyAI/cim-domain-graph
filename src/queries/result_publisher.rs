//! Simple query result publisher implementation
//!
//! This module provides a trait and implementation for publishing query results
//! with correlation tracking.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

/// Trait for publishing query results with correlation
#[async_trait]
pub trait QueryResultPublisher: Send + Sync {
    /// Publish a query result
    async fn publish_result(
        &self,
        query_id: &str,
        query_type: &str,
        result: &Value,
        correlation_id: String,
        causation_id: Option<String>,
        user_id: String,
    ) -> Result<(), String>;
}

/// A simple logging implementation of QueryResultPublisher
pub struct LoggingQueryResultPublisher;

#[async_trait]
impl QueryResultPublisher for LoggingQueryResultPublisher {
    async fn publish_result(
        &self,
        query_id: &str,
        query_type: &str,
        result: &Value,
        correlation_id: String,
        causation_id: Option<String>,
        user_id: String,
    ) -> Result<(), String> {
        info!(
            query_id = %query_id,
            query_type = %query_type,
            correlation_id = %correlation_id,
            causation_id = ?causation_id,
            user_id = %user_id,
            result_size = result.to_string().len(),
            "Publishing query result with correlation"
        );
        Ok(())
    }
}

/// Factory function to create a query result publisher
pub fn create_query_result_publisher() -> Arc<dyn QueryResultPublisher> {
    Arc::new(LoggingQueryResultPublisher)
}

/// Helper macro to publish query results
#[macro_export]
macro_rules! publish_query_result {
    ($publisher:expr, $envelope:expr, $query_type:expr, $result:expr) => {
        if let Ok(result_value) = serde_json::to_value(&$result) {
            let _ = $publisher.publish_result(
                &$envelope.id.to_string(),
                $query_type,
                &result_value,
                $envelope.correlation_id().to_string(),
                Some($envelope.id.to_string()),
                $envelope.issued_by.clone(),
            ).await;
        }
    };
}