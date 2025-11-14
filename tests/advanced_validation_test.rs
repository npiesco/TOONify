use std::process::Command;
use std::fs;

fn get_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/toonify", manifest_dir)
}

#[test]
fn test_validate_with_regex_pattern() {
    println!("=== Advanced Validation: Regex pattern matching ===");
    
    let schema = r#"{
  "users": {
    "type": "array",
    "fields": ["id", "email", "phone"],
    "field_types": {
      "id": "number",
      "email": "string",
      "phone": "string"
    },
    "patterns": {
      "email": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
      "phone": "^\\d{3}-\\d{3}-\\d{4}$"
    }
  }
}"#;
    
    // Valid data matching patterns
    let toon_data = r#"users[2]{id,email,phone}:
1,alice@example.com,555-123-4567
2,bob@test.org,555-987-6543"#;
    
    let schema_file = "/tmp/advanced_schema_regex.json";
    let toon_file = "/tmp/advanced_regex.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation should succeed for valid patterns");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Regex pattern validation successful\n");
}

#[test]
fn test_validate_with_invalid_regex_pattern() {
    println!("=== Advanced Validation: Invalid regex pattern ===");
    
    let schema = r#"{
  "contacts": {
    "type": "array",
    "fields": ["name", "email"],
    "field_types": {
      "name": "string",
      "email": "string"
    },
    "patterns": {
      "email": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
    }
  }
}"#;
    
    // Invalid email format
    let toon_data = r#"contacts[2]{name,email}:
Alice,alice@example.com
Bob,not-an-email"#;
    
    let schema_file = "/tmp/advanced_schema_invalid_regex.json";
    let toon_file = "/tmp/advanced_invalid_regex.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(!output.status.success(), "Validation should fail for invalid pattern");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pattern") || stderr.contains("email") || stderr.contains("match"),
            "Error should mention pattern mismatch");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Invalid pattern validation failed as expected\n");
}

#[test]
fn test_validate_with_number_range() {
    println!("=== Advanced Validation: Number range constraints ===");
    
    let schema = r#"{
  "products": {
    "type": "array",
    "fields": ["id", "price", "rating"],
    "field_types": {
      "id": "number",
      "price": "number",
      "rating": "number"
    },
    "ranges": {
      "price": {"min": 0.01, "max": 9999.99},
      "rating": {"min": 1, "max": 5}
    }
  }
}"#;
    
    // Valid data within ranges
    let toon_data = r#"products[3]{id,price,rating}:
1,19.99,4.5
2,149.50,5
3,0.99,3"#;
    
    let schema_file = "/tmp/advanced_schema_range.json";
    let toon_file = "/tmp/advanced_range.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation should succeed for valid ranges");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Number range validation successful\n");
}

#[test]
fn test_validate_with_out_of_range_number() {
    println!("=== Advanced Validation: Out of range number ===");
    
    let schema = r#"{
  "scores": {
    "type": "array",
    "fields": ["player", "score"],
    "field_types": {
      "player": "string",
      "score": "number"
    },
    "ranges": {
      "score": {"min": 0, "max": 100}
    }
  }
}"#;
    
    // Score exceeds maximum
    let toon_data = r#"scores[2]{player,score}:
Alice,95
Bob,150"#;
    
    let schema_file = "/tmp/advanced_schema_out_of_range.json";
    let toon_file = "/tmp/advanced_out_of_range.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(!output.status.success(), "Validation should fail for out of range value");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("range") || stderr.contains("max") || stderr.contains("score"),
            "Error should mention range violation");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Out of range validation failed as expected\n");
}

#[test]
fn test_validate_with_string_length() {
    println!("=== Advanced Validation: String length constraints ===");
    
    let schema = r#"{
  "users": {
    "type": "array",
    "fields": ["username", "password", "bio"],
    "field_types": {
      "username": "string",
      "password": "string",
      "bio": "string"
    },
    "string_lengths": {
      "username": {"min": 3, "max": 20},
      "password": {"min": 8},
      "bio": {"max": 500}
    }
  }
}"#;
    
    // Valid string lengths
    let toon_data = r#"users[2]{username,password,bio}:
alice123,SecurePass123,Software developer
bob456,MyPassword2024,Designer and artist"#;
    
    let schema_file = "/tmp/advanced_schema_length.json";
    let toon_file = "/tmp/advanced_length.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation should succeed for valid string lengths");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ String length validation successful\n");
}

#[test]
fn test_validate_with_invalid_string_length() {
    println!("=== Advanced Validation: Invalid string length ===");
    
    let schema = r#"{
  "accounts": {
    "type": "array",
    "fields": ["username", "password"],
    "field_types": {
      "username": "string",
      "password": "string"
    },
    "string_lengths": {
      "username": {"min": 5, "max": 15},
      "password": {"min": 10}
    }
  }
}"#;
    
    // Username too short (2 chars, needs 5 min)
    let toon_data = r#"accounts[1]{username,password}:
ab,ValidPassword123"#;
    
    let schema_file = "/tmp/advanced_schema_invalid_length.json";
    let toon_file = "/tmp/advanced_invalid_length.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(!output.status.success(), "Validation should fail for invalid string length");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("length") || stderr.contains("min") || stderr.contains("username"),
            "Error should mention string length violation");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Invalid string length validation failed as expected\n");
}

#[test]
fn test_validate_with_enum_values() {
    println!("=== Advanced Validation: Enum value constraints ===");
    
    let schema = r#"{
  "orders": {
    "type": "array",
    "fields": ["id", "status", "priority"],
    "field_types": {
      "id": "number",
      "status": "string",
      "priority": "string"
    },
    "enums": {
      "status": ["pending", "processing", "shipped", "delivered", "cancelled"],
      "priority": ["low", "medium", "high", "urgent"]
    }
  }
}"#;
    
    // Valid enum values
    let toon_data = r#"orders[3]{id,status,priority}:
1,pending,high
2,shipped,medium
3,delivered,low"#;
    
    let schema_file = "/tmp/advanced_schema_enum.json";
    let toon_file = "/tmp/advanced_enum.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation should succeed for valid enum values");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Enum validation successful\n");
}

#[test]
fn test_validate_with_invalid_enum_value() {
    println!("=== Advanced Validation: Invalid enum value ===");
    
    let schema = r#"{
  "tasks": {
    "type": "array",
    "fields": ["id", "status"],
    "field_types": {
      "id": "number",
      "status": "string"
    },
    "enums": {
      "status": ["todo", "in-progress", "done"]
    }
  }
}"#;
    
    // Invalid status value
    let toon_data = r#"tasks[2]{id,status}:
1,todo
2,completed"#;
    
    let schema_file = "/tmp/advanced_schema_invalid_enum.json";
    let toon_file = "/tmp/advanced_invalid_enum.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(!output.status.success(), "Validation should fail for invalid enum value");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("enum") || stderr.contains("allowed") || stderr.contains("status") || stderr.contains("completed"),
            "Error should mention enum violation");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Invalid enum validation failed as expected\n");
}

#[test]
fn test_validate_with_custom_format() {
    println!("=== Advanced Validation: Custom format validation ===");
    
    let schema = r#"{
  "contacts": {
    "type": "array",
    "fields": ["name", "email", "website", "created"],
    "field_types": {
      "name": "string",
      "email": "string",
      "website": "string",
      "created": "string"
    },
    "formats": {
      "email": "email",
      "website": "url",
      "created": "date"
    }
  }
}"#;
    
    // Valid formatted data
    let toon_data = r#"contacts[2]{name,email,website,created}:
Alice,alice@example.com,https://alice.com,2024-01-15
Bob,bob@test.org,https://bob.test.org,2024-02-20"#;
    
    let schema_file = "/tmp/advanced_schema_format.json";
    let toon_file = "/tmp/advanced_format.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation should succeed for valid formats");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Custom format validation successful\n");
}

#[test]
fn test_validate_with_invalid_format() {
    println!("=== Advanced Validation: Invalid format ===");
    
    let schema = r#"{
  "users": {
    "type": "array",
    "fields": ["name", "email"],
    "field_types": {
      "name": "string",
      "email": "string"
    },
    "formats": {
      "email": "email"
    }
  }
}"#;
    
    // Invalid email format
    let toon_data = r#"users[2]{name,email}:
Alice,alice@example.com
Bob,invalid-email-format"#;
    
    let schema_file = "/tmp/advanced_schema_invalid_format.json";
    let toon_file = "/tmp/advanced_invalid_format.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(!output.status.success(), "Validation should fail for invalid format");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("format") || stderr.contains("email") || stderr.contains("invalid"),
            "Error should mention format violation");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Invalid format validation failed as expected\n");
}

