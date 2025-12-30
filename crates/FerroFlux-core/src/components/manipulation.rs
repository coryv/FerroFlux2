use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Helper component to split a list into individual items (Fan-Out).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SplitConfig {
    /// The path to the array field to split. If None, splits the root object if it is an array.
    pub path: Option<String>,
}

/// Component for aggregating multiple inputs into a batch (Fan-In).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AggregateConfig {
    /// Number of items to wait for before emitting a batch.
    pub batch_size: usize,
    /// Max time to wait before emitting a partial batch.
    pub timeout_seconds: u64,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub expression: String,
    pub result_key: Option<String>,
}

/// Configuration for statistical analysis of a stream.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    /// The numeric field to analyze.
    pub target_field: String,
    /// Where to write the stats.
    pub enrichment_key: String,
    /// Whether to flag outliers.
    pub detect_outliers: bool,
    /// Z-score threshold for outliers.
    pub threshold: f64,
}

/// Operations available for Sliding Window analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowOp {
    /// Calculate the mean/average.
    Mean,
    /// Calculate the sum.
    Sum,
    /// Find the minimum value.
    Min,
    /// Find the maximum value.
    Max,
    /// Calculate the variance.
    Variance,
}

/// State management for sliding windows.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Number of samples to keep.
    pub window_size: usize,
    /// The operation to perform on the window.
    pub operation: WindowOp,
    /// The input field.
    pub target_field: String,
    /// The output field.
    pub result_key: String,
}

#[derive(Component, Debug, Default)]
pub struct WindowState {
    pub buffer: std::collections::VecDeque<f64>,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionConfig {
    pub expression: String,
    pub result_key: String,
}

#[derive(Component, Debug, Clone, Default)]
pub struct BatchState {
    pub items: Vec<serde_json::Value>,
    pub last_update: Option<std::time::Instant>,
}
