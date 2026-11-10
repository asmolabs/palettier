use serde_json::{Map, Value};

/// Adapts a JSON schema for OpenAI strict mode (`additionalProperties: false` and full `required` on every object).
pub fn strict_schema(schema: &Value) -> Value {
    match schema {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k.clone(), strict_schema(v));
            }
            if map.get("type").and_then(Value::as_str) == Some("object") {
                let props = map
                    .get("properties")
                    .and_then(Value::as_object)
                    .map(|p| p.keys().cloned().map(Value::String).collect::<Vec<_>>())
                    .unwrap_or_default();
                out.insert("required".to_string(), Value::Array(props));
                out.insert("additionalProperties".to_string(), Value::Bool(false));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(strict_schema).collect()),
        other => other.clone(),
    }
}

/// Adapts a JSON schema to the OpenAPI subset expected by Gemini (`responseSchema`), removing `$schema` and `additionalProperties`.
pub fn open_api_schema(schema: &Value) -> Value {
    match schema {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                if k == "$schema" || k == "additionalProperties" {
                    continue;
                }
                out.insert(k.clone(), open_api_schema(v));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(open_api_schema).collect()),
        other => other.clone(),
    }
}
