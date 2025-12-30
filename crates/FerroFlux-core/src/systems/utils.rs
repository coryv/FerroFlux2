use serde_json::Value;

pub fn merge_result(
    original_input: &Value,
    new_result: &str, // Or Value, but typically we get string results from LLM/Script
    result_key: Option<&String>,
) -> String {
    match result_key {
        Some(key) => {
            // Enrichment Mode: Merge result into original input
            let mut input_clone = original_input.clone();

            // Try to parse result as JSON, otherwise treat as string
            let result_value = serde_json::from_str(new_result)
                .unwrap_or_else(|_| Value::String(new_result.to_string()));

            // Ensure input is an Object to insert into
            if let Some(obj) = input_clone.as_object_mut() {
                obj.insert(key.clone(), result_value);
                serde_json::to_string(&input_clone).unwrap_or(new_result.to_string())
            } else {
                // If input wasn't an object, we can't key into it easily.
                // Fallback: Return just the result or maybe wrap input?
                // For now, if input is not object, just return result or log warning?
                // Defaulting to replacing content if input is not an object is safer for now.
                new_result.to_string()
            }
        }
        None => {
            // Legacy Mode: Replace input with output
            new_result.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_result_none() {
        let input = json!({"hello": "world"});
        let result = "replacement";
        // Legacy behavior: Replace input entirely
        let output = merge_result(&input, result, None);
        assert_eq!(output, "replacement");
    }

    #[test]
    fn test_merge_result_some() {
        let input = json!({"hello": "world"});
        let result = "enriched";
        let key = "new_field".to_string();

        // Enrichment: Input becomes {"hello": "world", "new_field": "enriched"}
        let output = merge_result(&input, result, Some(&key));
        let output_json: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(output_json["hello"], "world");
        assert_eq!(output_json["new_field"], "enriched");
    }

    #[test]
    fn test_merge_result_json_value() {
        let input = json!({"data": [1, 2]});
        let result_json = r#"{"analysis": "good"}"#;
        let key = "meta".to_string();

        let output = merge_result(&input, result_json, Some(&key));
        let output_json: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(output_json["data"][0], 1);
        assert_eq!(output_json["meta"]["analysis"], "good");
    }
}
