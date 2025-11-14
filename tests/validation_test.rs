use std::process::Command;
use std::fs;

fn get_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/toonify", manifest_dir)
}

#[test]
fn test_validate_toon_with_valid_schema() {
    println!("=== Validation: Valid TOON against schema ===");
    
    // Create schema file
    let schema = r#"{
  "users": {
    "type": "array",
    "fields": ["id", "name", "email"],
    "field_types": {
      "id": "number",
      "name": "string",
      "email": "string"
    }
  }
}"#;
    
    // Create valid TOON data
    let toon_data = r#"users[3]{id,name,email}:
1,Alice,alice@example.com
2,Bob,bob@example.com
3,Charlie,charlie@example.com"#;
    
    let schema_file = "/tmp/test_schema.json";
    let toon_file = "/tmp/test_valid_toon.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    // Validate
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation should succeed for valid TOON");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid") || stdout.contains("passed") || stdout.contains("✓"), 
            "Output should indicate validation success");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Valid TOON validation successful\n");
}

#[test]
fn test_validate_toon_with_missing_field() {
    println!("=== Validation: TOON with missing required field ===");
    
    let schema = r#"{
  "products": {
    "type": "array",
    "fields": ["id", "name", "price", "category"],
    "field_types": {
      "id": "number",
      "name": "string",
      "price": "number",
      "category": "string"
    }
  }
}"#;
    
    // TOON missing 'category' field
    let toon_data = r#"products[2]{id,name,price}:
1,Widget,19.99
2,Gadget,29.99"#;
    
    let schema_file = "/tmp/test_schema_missing.json";
    let toon_file = "/tmp/test_missing_field.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Should fail validation
    assert!(!output.status.success(), "Validation should fail for missing field");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("category") || stderr.contains("missing") || stderr.contains("field"),
            "Error should mention missing field");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Missing field validation failed as expected\n");
}

#[test]
fn test_validate_toon_with_wrong_type() {
    println!("=== Validation: TOON with wrong field type ===");
    
    let schema = r#"{
  "inventory": {
    "type": "array",
    "fields": ["sku", "quantity", "price"],
    "field_types": {
      "sku": "string",
      "quantity": "number",
      "price": "number"
    }
  }
}"#;
    
    // 'quantity' should be number but is string
    let toon_data = r#"inventory[2]{sku,quantity,price}:
SKU001,not-a-number,19.99
SKU002,50,29.99"#;
    
    let schema_file = "/tmp/test_schema_type.json";
    let toon_file = "/tmp/test_wrong_type.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Should fail validation
    assert!(!output.status.success(), "Validation should fail for wrong type");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("type") || stderr.contains("number") || stderr.contains("quantity"),
            "Error should mention type mismatch");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Wrong type validation failed as expected\n");
}

#[test]
fn test_validate_toon_via_stdin() {
    println!("=== Validation: TOON via stdin ===");
    
    let schema = r#"{
  "data": {
    "type": "array",
    "fields": ["id", "value"],
    "field_types": {
      "id": "number",
      "value": "string"
    }
  }
}"#;
    
    let toon_data = "data[2]{id,value}:\n1,test\n2,example";
    
    let schema_file = "/tmp/test_schema_stdin.json";
    fs::write(schema_file, schema).expect("Failed to write schema");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(toon_data.as_bytes())?;
            child.wait_with_output()
        })
        .expect("Failed to validate via stdin");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Validation via stdin should succeed");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    
    println!("✓ stdin validation successful\n");
}

#[test]
fn test_validate_toon_array_count() {
    println!("=== Validation: Array count validation ===");
    
    let schema = r#"{
  "users": {
    "type": "array",
    "min_items": 3,
    "max_items": 5,
    "fields": ["id", "name"],
    "field_types": {
      "id": "number",
      "name": "string"
    }
  }
}"#;
    
    // Only 2 items but schema requires min 3
    let toon_data = r#"users[2]{id,name}:
1,Alice
2,Bob"#;
    
    let schema_file = "/tmp/test_schema_count.json";
    let toon_file = "/tmp/test_count.toon";
    
    fs::write(schema_file, schema).expect("Failed to write schema");
    fs::write(toon_file, toon_data).expect("Failed to write TOON data");
    
    let output = Command::new(get_binary_path())
        .args(&["validate", "--schema", schema_file, "--input", toon_file])
        .output()
        .expect("Failed to execute validate command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Should fail validation
    assert!(!output.status.success(), "Validation should fail for insufficient items");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("min") || stderr.contains("items") || stderr.contains("count"),
            "Error should mention minimum item count");
    
    // Cleanup
    let _ = fs::remove_file(schema_file);
    let _ = fs::remove_file(toon_file);
    
    println!("✓ Array count validation failed as expected\n");
}

