use crate::tools::{Tool, ToolContext};
use anyhow::Result;
use serde_json::{Value, json};

pub struct StatsTool;

impl Tool for StatsTool {
    fn id(&self) -> &'static str {
        "ferroflux:stats"
    }

    fn run(&self, _ctx: &mut ToolContext, params: Value) -> Result<Value> {
        let target_field = params
            .get("target_field")
            .and_then(|v| v.as_str())
            .unwrap_or("value");
        let enrichment_key = params
            .get("enrichment_key")
            .and_then(|v| v.as_str())
            .unwrap_or("stats");
        let detect_outliers = params
            .get("detect_outliers")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let threshold = params
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0);

        // Determine data source: "data" param or context array
        let data_array = if let Some(data) = params.get("data").and_then(|v| v.as_array()) {
            data.clone()
        } else {
            // If no data param, check if the "current context" is an array?
            // Tools usually operate on inputs. But `stats_worker` operated on the *Payload*.
            // In the new model, nodes pass specific data via params.
            // So we expect `data` to be the array.
            // But for compatibility or ease, let's treat "data" as required if we want to analyze it.
            // Or maybe we treat `params` itself as the object if it has an array?
            return Err(anyhow::anyhow!(
                "StatsTool requires 'data' parameter to be an array"
            ));
        };

        // Pass 1: Collect Values
        let mut values: Vec<f64> = Vec::new();
        for item in &data_array {
            if let Some(val) = item.get(target_field).and_then(|v| v.as_f64()) {
                values.push(val);
            } else if let Some(obj) = item.as_object() {
                if let Some(val) = obj.get(target_field).and_then(|v| v.as_f64()) {
                    values.push(val);
                }
            }
        }

        let count = values.len() as f64;
        if count == 0.0 {
            return Ok(Value::Array(data_array));
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|v| {
                let diff = mean - *v;
                diff * diff
            })
            .sum::<f64>()
            / count;
        let std_dev = variance.sqrt();

        // Pass 2: Enrich
        let mut enriched_array = data_array.clone();
        let mut outlier_count = 0;

        for item in enriched_array.iter_mut() {
            // Access mutable object. If item is not object, we can't enrich it directly unless we wrap it?
            // Assuming array of objects.
            if let Some(obj) = item.as_object_mut() {
                let val_opt = obj.get(target_field).and_then(|v| v.as_f64());

                if let Some(val) = val_opt {
                    let z_score = if std_dev == 0.0 {
                        0.0
                    } else {
                        (val - mean) / std_dev
                    };
                    let is_outlier = detect_outliers && z_score.abs() > threshold;

                    if is_outlier {
                        outlier_count += 1;
                    }
                    // Telemetry could use outlier_count
                    let _ = outlier_count;

                    let stats_obj = json!({
                        "mean": mean,
                        "std_dev": std_dev,
                        "z_score": z_score,
                        "is_outlier": is_outlier
                    });

                    obj.insert(enrichment_key.to_string(), stats_obj);
                }
            }
        }

        // Also return summary stats?
        // The tool returns `Value`. It could return { "data": enriched_array, "summary": ... }
        // For compatibility with "piping", passing the array through (enriched) is best.
        // We can ALSO write summary to context if needed, but tool return is usually the "output".

        Ok(Value::Array(enriched_array))
    }
}
