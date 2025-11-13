use serde_json::{Value, Map, Number};

pub fn parse_toon(input: &str) -> Result<Value, String> {
    let mut root = Map::new();
    let lines: Vec<&str> = input.lines().map(|l| l.trim_end()).collect();
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        
        if let Some((key, rest)) = parse_header_line(line) {
            let (value, consumed) = parse_value(&lines[i..], rest)?;
            root.insert(key, value);
            i += consumed;
        } else {
            i += 1;
        }
    }
    
    Ok(Value::Object(root))
}

fn parse_header_line(line: &str) -> Option<(String, HeaderInfo)> {
    if !line.contains(':') {
        return None;
    }
    
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let header = parts[0];
    let mut key = String::new();
    let mut is_array = false;
    let mut count = 0;
    let mut columns = Vec::new();
    
    if let Some(array_start) = header.find('[') {
        key = header[..array_start].to_string();
        is_array = true;
        
        if let Some(array_end) = header.find(']') {
            if let Ok(c) = header[array_start + 1..array_end].parse::<usize>() {
                count = c;
            }
        }
        
        if let Some(cols_start) = header.find('{') {
            if let Some(cols_end) = header.find('}') {
                let cols_str = &header[cols_start + 1..cols_end];
                columns = cols_str.split(',').map(|s| s.trim().to_string()).collect();
            }
        }
    } else if let Some(cols_start) = header.find('{') {
        key = header[..cols_start].to_string();
        if let Some(cols_end) = header.find('}') {
            let cols_str = &header[cols_start + 1..cols_end];
            columns = cols_str.split(',').map(|s| s.trim().to_string()).collect();
        }
    } else {
        key = header.to_string();
    }
    
    Some((key, HeaderInfo { is_array, count, columns }))
}

struct HeaderInfo {
    is_array: bool,
    count: usize,
    columns: Vec<String>,
}

fn parse_value(lines: &[&str], info: HeaderInfo) -> Result<(Value, usize), String> {
    if info.is_array || !info.columns.is_empty() {
        let mut array_items = Vec::new();
        let mut consumed = 1;
        
        for i in 1..lines.len() {
            let line = lines[i].trim();
            
            if line.is_empty() {
                break;
            }
            
            if line.contains(':') && !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            
            if !info.columns.is_empty() {
                let mut obj = Map::new();
                let values = split_csv_line(line);
                
                for (idx, col) in info.columns.iter().enumerate() {
                    if idx < values.len() {
                        obj.insert(col.clone(), parse_csv_value(&values[idx]));
                    }
                }
                
                array_items.push(Value::Object(obj));
            } else {
                let values = split_csv_line(line);
                for v in &values {
                    array_items.push(parse_csv_value(v));
                }
            }
            
            consumed += 1;
        }
        
        Ok((Value::Array(array_items), consumed))
    } else {
        let rest = if lines[0].contains(':') {
            lines[0].split(':').nth(1).unwrap_or("").trim()
        } else {
            ""
        };
        
        Ok((parse_csv_value(rest), 1))
    }
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    
    if !current.is_empty() || !parts.is_empty() {
        parts.push(current.trim().to_string());
    }
    
    parts
}

fn parse_csv_value(s: &str) -> Value {
    let s = s.trim();
    
    if s.is_empty() {
        return Value::Null;
    }
    
    if s == "null" {
        return Value::Null;
    }
    
    if s == "true" {
        return Value::Bool(true);
    }
    
    if s == "false" {
        return Value::Bool(false);
    }
    
    if let Ok(num) = s.parse::<i64>() {
        return Value::Number(Number::from(num));
    }
    
    if let Ok(num) = s.parse::<f64>() {
        if let Some(n) = Number::from_f64(num) {
            return Value::Number(n);
        }
    }
    
    Value::String(s.to_string())
}
