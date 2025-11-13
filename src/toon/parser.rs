use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::{char, digit1, multispace0, newline},
    combinator::{map, map_res, opt},
    multi::{many0, separated_list0},
    sequence::{preceded, terminated},
    IResult,
};
use serde_json::{Map, Number, Value};

pub fn parse_toon(input: &str) -> Result<Value, String> {
    match toon_document(input) {
        Ok((remaining, value)) => {
            if !remaining.trim().is_empty() {
                return Err(format!("Parse error: unexpected content at end: {:?}", remaining.chars().take(50).collect::<String>()));
            }
            Ok(value)
        },
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

fn toon_document(input: &str) -> IResult<&str, Value> {
    let (input, _) = multispace0(input)?;
    let (input, entries) = many0(terminated(entry, multispace0))(input)?;
    
    let mut map = Map::new();
    for (key, value) in entries {
        map.insert(key, value);
    }
    
    Ok((input, Value::Object(map)))
}

fn entry(input: &str) -> IResult<&str, (String, Value)> {
    let (input, key) = identifier(input)?;
    let (input, meta) = opt(metadata)(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, value) = if let Some((is_array, columns)) = meta {
        if is_array {
            array_value(input, columns)?
        } else if !columns.is_empty() {
            object_value(input, columns)?
        } else {
            let (input, rest) = take_until_newline_or_end(input)?;
            let val = parse_value(rest.trim());
            (input, val)
        }
    } else {
        let (input, rest) = take_until_newline_or_end(input)?;
        let val = parse_value(rest.trim());
        (input, val)
    };
    
    Ok((input, (key.to_string(), value)))
}

fn metadata(input: &str) -> IResult<&str, (bool, Vec<String>)> {
    let (input, array_meta) = opt(array_metadata)(input)?;
    let (input, columns) = opt(column_metadata)(input)?;
    
    let is_array = array_meta.is_some();
    let cols = columns.unwrap_or_default();
    
    Ok((input, (is_array, cols)))
}

fn array_metadata(input: &str) -> IResult<&str, usize> {
    let (input, _) = char('[')(input)?;
    let (input, count) = map_res(digit1, |s: &str| s.parse::<usize>())(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, count))
}

fn column_metadata(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = char('{')(input)?;
    let (input, cols) = separated_list0(
        char(','),
        map(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |s: &str| s.trim().to_string(),
        ),
    )(input)?;
    let (input, _) = char('}')(input)?;
    Ok((input, cols))
}

fn array_value(input: &str, columns: Vec<String>) -> IResult<&str, Value> {
    let (input, lines) = many0(preceded(multispace0, data_line))(input)?;
    
    let mut items = Vec::new();
    
    for line in lines {
        if !columns.is_empty() {
            let values = split_csv(&line);
            let mut obj = Map::new();
            
            for (idx, col) in columns.iter().enumerate() {
                if idx < values.len() {
                    obj.insert(col.clone(), parse_value(&values[idx]));
                }
            }
            
            items.push(Value::Object(obj));
        } else {
            let values = split_csv(&line);
            for v in values {
                items.push(parse_value(&v));
            }
        }
    }
    
    Ok((input, Value::Array(items)))
}

fn object_value(input: &str, columns: Vec<String>) -> IResult<&str, Value> {
    let (input, _) = multispace0(input)?;
    let (input, line) = data_line(input)?;
    
    let values = split_csv(&line);
    let mut obj = Map::new();
    
    for (idx, col) in columns.iter().enumerate() {
        if idx < values.len() {
            obj.insert(col.clone(), parse_value(&values[idx]));
        }
    }
    
    Ok((input, Value::Object(obj)))
}

fn data_line(input: &str) -> IResult<&str, String> {
    let (input, line) = take_until_newline_or_end(input)?;
    let trimmed = line.trim();
    
    if trimmed.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    
    if looks_like_entry_header(trimmed) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    
    Ok((input, line.to_string()))
}

fn looks_like_entry_header(line: &str) -> bool {
    if let Some(colon_pos) = line.find(':') {
        let before_colon = &line[..colon_pos];
        let chars: Vec<char> = before_colon.chars().collect();
        
        let mut i = 0;
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        
        if i == 0 {
            return false;
        }
        
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        
        if i == chars.len() {
            return true;
        }
        
        if i < chars.len() && (chars[i] == '[' || chars[i] == '{') {
            return true;
        }
    }
    
    false
}

fn take_until_newline_or_end(input: &str) -> IResult<&str, &str> {
    let (remaining, content) = take_while(|c| c != '\n' && c != '\r')(input)?;
    let (remaining, _) = opt(newline)(remaining)?;
    Ok((remaining, content))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn split_csv(line: &str) -> Vec<String> {
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

fn parse_value(s: &str) -> Value {
    let s = s.trim();
    
    if s.is_empty() || s == "null" {
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
