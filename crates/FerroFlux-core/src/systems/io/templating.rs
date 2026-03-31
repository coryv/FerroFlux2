use cel_interpreter::{Context, Program, Value as CelValue};
use serde_json::Value;
use std::sync::Arc;

pub fn json_to_cel(v: Value) -> CelValue {
    match v {
        Value::Null => CelValue::Null,
        Value::Bool(b) => CelValue::Bool(b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CelValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                CelValue::Float(f)
            } else {
                CelValue::Null
            }
        }
        Value::String(s) => CelValue::String(Arc::new(s)),
        Value::Array(arr) => {
            let cel_arr: Vec<CelValue> = arr.into_iter().map(json_to_cel).collect();
            CelValue::List(Arc::new(cel_arr))
        }
        Value::Object(obj) => {
            let mut cel_map = std::collections::HashMap::new();
            for (k, v) in obj {
                cel_map.insert(Arc::new(k), json_to_cel(v));
            }
            CelValue::Map(cel_map.into())
        }
    }
}

pub fn cel_to_json(v: CelValue) -> Value {
    match v {
        CelValue::Null => Value::Null,
        CelValue::Bool(b) => Value::Bool(b),
        CelValue::Int(i) => Value::Number(i.into()),
        CelValue::UInt(u) => Value::Number(u.into()),
        CelValue::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        CelValue::String(s) => Value::String(s.to_string()),
        CelValue::Bytes(b) => Value::String(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &*b,
        )),
        CelValue::List(l) => {
            let arr: Vec<Value> = l.iter().cloned().map(cel_to_json).collect();
            Value::Array(arr)
        }
        CelValue::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m.map.iter() {
                obj.insert(k.to_string(), cel_to_json(v.clone()));
            }
            Value::Object(obj)
        }
        _ => Value::Null,
    }
}

fn json_cel_func(v: CelValue) -> Arc<String> {
    let json_val = cel_to_json(v);
    Arc::new(serde_json::to_string(&json_val).unwrap_or_else(|_| "null".to_string()))
}

pub fn apply_template(template: &str, json: &serde_json::Value) -> String {
    if template.trim().is_empty() {
        return template.to_string();
    }
    let program = match Program::compile(template) {
        Ok(p) => p,
        Err(e) => return format!("Template Compilation Error: {}", e),
    };

    let mut context = Context::default();

    // Register custom functions
    context.add_function("json", json_cel_func);

    // Load context data
    if let Some(obj) = json.as_object() {
        for (k, v) in obj {
            let _ = context.add_variable(k, json_to_cel(v.clone()));
        }
    }

    match program.execute(&context) {
        Ok(val) => match val {
            CelValue::String(s) => s.to_string(),
            _ => {
                let json_val = cel_to_json(val);
                if json_val.is_string() {
                    json_val.as_str().unwrap().to_string()
                } else {
                    json_val.to_string()
                }
            }
        },
        Err(e) => format!("Template Execution Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_apply_template_simple() {
        // CEL: string concatenation
        let template = r#"'{"msg": "' + text + '", "channel": "' + chan + '"}'"#;
        let input = serde_json::json!({
            "text": "Hello World",
            "chan": "general"
        });

        let result_str = apply_template(template, &input);

        assert_eq!(
            result_str,
            r#"{"msg": "Hello World", "channel": "general"}"#
        );
    }

    #[test]
    fn test_apply_template_cel_logic() {
        let template = r#"{
            "system": {
                "parts": [ { "text": json(system_instruction) } ]
            },
            "is_user": role == "user"
        }"#;

        let input = serde_json::json!({
            "system_instruction": "Be helpful.",
            "role": "user"
        });

        let result_str = apply_template(template, &input);

        let parsed: serde_json::Value =
            serde_json::from_str(&result_str).expect("Result should be valid JSON");
        assert_eq!(parsed["system"]["parts"][0]["text"], "\"Be helpful.\"");
        assert_eq!(parsed["is_user"], true);
    }

    #[test]
    fn test_apply_template_complex_cel() {
        let template_simple = r#"role == 'admin' ? 'ALLOWED' : 'DENIED'"#;
        assert_eq!(apply_template(template_simple, &json!({"role": "admin"})), "ALLOWED");
        assert_eq!(apply_template(template_simple, &json!({"role": "user"})), "DENIED");
    }
}
