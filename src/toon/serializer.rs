use serde_json::Value;

pub fn serialize_toon(value: &Value) -> Result<String, String> {
    match value {
        Value::Object(map) => {
            let mut output = String::new();
            
            for (key, val) in map {
                output.push_str(&serialize_entry(key, val)?);
                output.push('\n');
            }
            
            Ok(output.trim_end().to_string())
        }
        _ => Err("Root value must be an object".to_string()),
    }
}

fn serialize_entry(key: &str, value: &Value) -> Result<String, String> {
    match value {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(format!("{}[0]:\n", key));
            }
            
            if let Some(Value::Object(first_obj)) = arr.first() {
                let columns: Vec<String> = first_obj.keys().cloned().collect();
                let mut output = format!("{}[{}]{{{}}}:\n", 
                    key, 
                    arr.len(), 
                    columns.join(",")
                );
                
                for item in arr {
                    if let Value::Object(obj) = item {
                        let mut row_values = Vec::new();
                        for col in &columns {
                            let val = obj.get(col).unwrap_or(&Value::Null);
                            row_values.push(serialize_value(val));
                        }
                        output.push_str(&row_values.join(","));
                        output.push('\n');
                    }
                }
                
                Ok(output)
            } else {
                let mut output = format!("{}[{}]:\n", key, arr.len());
                for item in arr {
                    output.push_str(&serialize_value(item));
                    output.push('\n');
                }
                Ok(output)
            }
        }
        Value::Object(obj) => {
            let columns: Vec<String> = obj.keys().cloned().collect();
            let mut output = format!("{}{{{}}}:\n", key, columns.join(","));
            
            let mut values = Vec::new();
            for col in &columns {
                let val = obj.get(col).unwrap_or(&Value::Null);
                values.push(serialize_value(val));
            }
            output.push_str(&values.join(","));
            output.push('\n');
            
            Ok(output)
        }
        _ => {
            Ok(format!("{}:{}\n", key, serialize_value(value)))
        }
    }
}

fn serialize_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}
