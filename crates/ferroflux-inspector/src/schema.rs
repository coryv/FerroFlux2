use ferroflux_core::integrations::registry::InputDef;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Password,
    Number,
    Boolean,
    Select { options: Vec<String> },
    Code { language: String },
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualField {
    pub id: String,
    pub label: String,
    pub description: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<Value>,
}

impl VisualField {
    pub fn from_input_def(def: &InputDef) -> Self {
        let field_type = match def.field_type.as_str() {
            "string" if def.is_secret => FieldType::Password,
            "string" => FieldType::Text,
            "number" => FieldType::Number,
            "boolean" => FieldType::Boolean,
            "json" => FieldType::Json,
            "code" => FieldType::Code {
                language: "javascript".to_string(), // Default or detect from options
            },
            _ => FieldType::Text,
        };

        let field_type = if let Some(options) = &def.options {
            FieldType::Select {
                options: options.clone(),
            }
        } else {
            field_type
        };

        Self {
            id: def.name.clone(),
            label: capitalize(&def.name),
            description: def.description.clone(),
            field_type,
            required: def.required,
            default_value: def.default.clone(),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mapping_text_field() {
        let def = InputDef {
            name: "api_key".to_string(),
            field_type: "string".to_string(),
            required: true,
            description: "An API Key".to_string(),
            is_secret: true,
            default: None,
            options: None,
            dynamic_source: None,
        };

        let visual = VisualField::from_input_def(&def);
        assert_eq!(visual.id, "api_key");
        assert_eq!(visual.label, "Api_key");
        assert!(matches!(visual.field_type, FieldType::Password));
        assert!(visual.required);
    }

    #[test]
    fn test_mapping_select_field() {
        let def = InputDef {
            name: "model".to_string(),
            field_type: "string".to_string(),
            required: false,
            description: "The model to use".to_string(),
            is_secret: false,
            default: Some(json!("gpt-4")),
            options: Some(vec!["gpt-4".to_string(), "gpt-3.5".to_string()]),
            dynamic_source: None,
        };

        let visual = VisualField::from_input_def(&def);
        if let FieldType::Select { options } = visual.field_type {
            assert_eq!(options.len(), 2);
            assert_eq!(options[0], "gpt-4");
        } else {
            panic!("Expected FieldType::Select");
        }
        assert_eq!(visual.default_value, Some(json!("gpt-4")));
    }
}
