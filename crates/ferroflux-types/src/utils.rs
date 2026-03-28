use serde_json::Value;

pub fn merge_result(
    original_input: &Value,
    new_result: &str,
    result_key: Option<&String>,
) -> String {
    match result_key {
        Some(key) => {
            let mut input_clone = original_input.clone();
            let result_value = serde_json::from_str(new_result)
                .unwrap_or_else(|_| Value::String(new_result.to_string()));

            if let Some(obj) = input_clone.as_object_mut() {
                obj.insert(key.clone(), result_value);
                serde_json::to_string(&input_clone).unwrap_or(new_result.to_string())
            } else {
                new_result.to_string()
            }
        }
        None => new_result.to_string(),
    }
}
