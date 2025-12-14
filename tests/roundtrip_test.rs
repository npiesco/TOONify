use serde_json::Value;
use toonify::converter;
use moka::sync::Cache;
use std::time::Duration;

#[test]
fn test_json_to_toon_to_json_roundtrip() {
    let original_json = r#"{
  "users": [
    {
      "id": 1,
      "name": "Sreeni",
      "role": "admin",
      "email": "sreeni@example.com"
    },
    {
      "id": 2,
      "name": "Krishna",
      "role": "admin",
      "email": "krishna@example.com"
    },
    {
      "id": 3,
      "name": "Aaron",
      "role": "user",
      "email": "aaron@example.com"
    }
  ],
  "metadata": {
    "total": 3,
    "last_updated": "2024-01-15T10:30:00Z"
  }
}"#;

    println!("=== JSON → TOON → JSON Round-trip Test ===");
    println!("Original JSON:\n{}\n", original_json);

    let toon = converter::json_to_toon(original_json).expect("Failed to convert JSON to TOON");
    println!("Converted to TOON:\n{}\n", toon);

    let final_json = converter::toon_to_json(&toon).expect("Failed to convert TOON back to JSON");
    println!("Converted back to JSON:\n{}\n", final_json);

    let original: Value = serde_json::from_str(original_json).expect("Failed to parse original JSON");
    let final_value: Value = serde_json::from_str(&final_json).expect("Failed to parse final JSON");

    assert_eq!(original, final_value, "Round-trip JSON → TOON → JSON failed: values don't match");
    println!("✓ Round-trip successful: JSON → TOON → JSON\n");
}

#[test]
fn test_toon_to_json_to_toon_roundtrip() {
    let original_toon = r#"users[3]{email,id,name,role}:
sreeni@example.com,1,Sreeni,admin
krishna@example.com,2,Krishna,admin
aaron@example.com,3,Aaron,user

metadata{last_updated,total}:
2024-01-15T10:30:00Z,3"#;

    println!("=== TOON → JSON → TOON Round-trip Test ===");
    println!("Original TOON:\n{}\n", original_toon);

    let json = converter::toon_to_json(original_toon).expect("Failed to convert TOON to JSON");
    println!("Converted to JSON:\n{}\n", json);

    let final_toon = converter::json_to_toon(&json).expect("Failed to convert JSON back to TOON");
    println!("Converted back to TOON:\n{}\n", final_toon);

    let original_value: Value = converter::toon_to_json(original_toon).and_then(|j| serde_json::from_str(&j).map_err(|e| e.to_string())).expect("Failed to parse original TOON");
    let final_value: Value = converter::toon_to_json(&final_toon).and_then(|j| serde_json::from_str(&j).map_err(|e| e.to_string())).expect("Failed to parse final TOON");

    assert_eq!(original_value, final_value, "Round-trip TOON → JSON → TOON failed: values don't match");
    println!("✓ Round-trip successful: TOON → JSON → TOON\n");
}

#[test]
fn test_complex_data_roundtrip() {
    let complex_json = r#"{
  "products": [
    {
      "id": 1,
      "name": "Laptop",
      "price": 999.99,
      "inStock": true
    },
    {
      "id": 2,
      "name": "Mouse",
      "price": 29.99,
      "inStock": false
    }
  ],
  "config": {
    "version": "1.0.0",
    "timestamp": "2024-11-13T01:53:00Z",
    "enabled": true
  }
}"#;

    println!("=== Complex Data Round-trip Test ===");
    
    let toon = converter::json_to_toon(complex_json).expect("Failed to convert JSON to TOON");
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    
    let original: Value = serde_json::from_str(complex_json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    assert_eq!(original, final_value, "Complex data round-trip failed");
    println!("✓ Complex data round-trip successful\n");
}

#[test]
fn test_varied_fields_roundtrip() {
    let varied_json = r#"{
  "items": [
    {
      "id": 1,
      "name": "Apple"
    },
    {
      "id": 2,
      "name": "Banana",
      "color": "yellow"
    },
    {
      "id": 3,
      "name": "Cherry",
      "price": 2.5
    }
  ]
}"#;

    println!("=== Varied Object Fields Round-trip Test ===");
    println!("Note: TOON normalizes schemas - missing fields become null\n");
    
    let toon = converter::json_to_toon(varied_json).expect("Failed to convert JSON to TOON");
    println!("TOON with normalized schema:\n{}\n", toon);
    
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    println!("Back to JSON (normalized):\n{}\n", back_to_json);
    
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    if let Value::Object(obj) = &final_value {
        if let Some(Value::Array(items)) = obj.get("items") {
            assert_eq!(items.len(), 3, "Should have 3 items");
            
            for item in items {
                if let Value::Object(item_obj) = item {
                    assert!(item_obj.contains_key("id"), "All items should have id");
                    assert!(item_obj.contains_key("name"), "All items should have name");
                    assert!(item_obj.contains_key("color"), "All items should have color (normalized)");
                    assert!(item_obj.contains_key("price"), "All items should have price (normalized)");
                }
            }
        }
    }
    
    println!("✓ Varied fields normalized correctly\n");
}

#[test]
fn test_colon_bearing_values() {
    let json_with_colons = r#"{
  "logs": [
    {
      "id": 1,
      "message": "error:404",
      "status": "failed"
    },
    {
      "id": 2,
      "message": "note:success",
      "status": "ok"
    }
  ]
}"#;

    println!("=== Colon-Bearing Values Round-trip Test ===");
    
    let toon = converter::json_to_toon(json_with_colons).expect("Failed to convert JSON to TOON");
    println!("TOON with colon-bearing values:\n{}\n", toon);
    
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    println!("Back to JSON:\n{}\n", back_to_json);
    
    let original: Value = serde_json::from_str(json_with_colons).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    assert_eq!(original, final_value, "Colon-bearing values round-trip failed");
    println!("✓ Colon-bearing values round-trip successful\n");
}

#[test]
fn test_timestamps_with_colons() {
    let json_with_timestamps = r#"{
  "events": [
    {
      "id": 1,
      "timestamp": "2024-01-15T10:30:00Z",
      "event": "login"
    },
    {
      "id": 2,
      "timestamp": "2024-01-15T11:45:30Z",
      "event": "logout"
    }
  ]
}"#;

    println!("=== Timestamps with Colons Round-trip Test ===");
    
    let toon = converter::json_to_toon(json_with_timestamps).expect("Failed to convert JSON to TOON");
    println!("TOON with timestamps:\n{}\n", toon);
    
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    println!("Back to JSON:\n{}\n", back_to_json);
    
    let original: Value = serde_json::from_str(json_with_timestamps).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    assert_eq!(original, final_value, "Timestamps round-trip failed");
    println!("✓ Timestamps round-trip successful\n");
}

#[test]
fn test_package_json_special_chars_roundtrip() {
    // Reproduces bug: keys with hyphens, @, / in column metadata fail to parse
    // Error: "Parse error: unexpected content at end: dependencies{react,react-dom}:..."
    let package_json = r#"{
  "name": "my-app",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.66",
    "@vitejs/plugin-react": "^4.2.1"
  }
}"#;

    println!("=== Package.json Special Characters Round-trip Test ===");
    println!("Original JSON:\n{}\n", package_json);

    let toon = converter::json_to_toon(package_json).expect("Failed to convert JSON to TOON");
    println!("Converted to TOON:\n{}\n", toon);

    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON back to JSON");
    println!("Converted back to JSON:\n{}\n", back_to_json);

    let original: Value = serde_json::from_str(package_json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();

    assert_eq!(original, final_value, "Package.json round-trip failed");
    println!("✓ Package.json special characters round-trip successful\n");
}

#[test]
fn test_package_json_with_urls_roundtrip() {
    // Reproduces bug: URL values with colons (https://) fail to parse
    // Error: "Parse error: unexpected content at end: bugs{url}:..."
    let package_json = r#"{
  "name": "my-app",
  "version": "1.0.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/npiesco/TOONify.git"
  },
  "bugs": {
    "url": "https://github.com/npiesco/TOONify/issues"
  },
  "homepage": "https://github.com/npiesco/TOONify"
}"#;

    println!("=== Package.json with URLs Round-trip Test ===");
    println!("Original JSON:\n{}\n", package_json);

    let toon = converter::json_to_toon(package_json).expect("Failed to convert JSON to TOON");
    println!("Converted to TOON:\n{}\n", toon);

    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON back to JSON");
    println!("Converted back to JSON:\n{}\n", back_to_json);

    let original: Value = serde_json::from_str(package_json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();

    assert_eq!(original, final_value, "Package.json with URLs round-trip failed");
    println!("✓ Package.json with URLs round-trip successful\n");
}

#[test]
fn test_full_vscode_extension_package_json_roundtrip() {
    // Read the actual vscode-extension/package.json file
    let package_json = std::fs::read_to_string("vscode-extension/package.json")
        .expect("Failed to read vscode-extension/package.json");

    println!("=== Full VSCode Extension Package.json Round-trip Test ===");
    println!("Original JSON:\n{}\n", package_json);

    let toon = converter::json_to_toon(&package_json).expect("Failed to convert JSON to TOON");
    println!("Converted to TOON:\n{}\n", toon);

    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON back to JSON");
    println!("Converted back to JSON:\n{}\n", back_to_json);

    let original: Value = serde_json::from_str(&package_json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();

    assert_eq!(original, final_value, "Full VSCode extension package.json round-trip failed");
    println!("✓ Full VSCode extension package.json round-trip successful\n");
}

#[test]
fn test_nested_objects_with_arrays_roundtrip() {
    // Reproduces bug: nested objects containing arrays of objects get corrupted
    // The "contributes.commands" array of objects was being mangled
    let package_json = r#"{
  "name": "my-extension",
  "contributes": {
    "commands": [
      {
        "command": "ext.doSomething",
        "title": "Do Something"
      },
      {
        "command": "ext.doOther",
        "title": "Do Other"
      }
    ],
    "menus": {
      "editor/context": [
        {
          "when": "editorHasSelection",
          "command": "ext.doSomething"
        }
      ]
    }
  }
}"#;

    println!("=== Nested Objects with Arrays Round-trip Test ===");
    println!("Original JSON:\n{}\n", package_json);

    let toon = converter::json_to_toon(package_json).expect("Failed to convert JSON to TOON");
    println!("Converted to TOON:\n{}\n", toon);

    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON back to JSON");
    println!("Converted back to JSON:\n{}\n", back_to_json);

    let original: Value = serde_json::from_str(package_json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();

    assert_eq!(original, final_value, "Nested objects with arrays round-trip failed");
    println!("✓ Nested objects with arrays round-trip successful\n");
}

#[test]
fn test_roundtrip_with_moka_cache_and_tmp() {
    // Test that reads actual file, caches original content with moka,
    // roundtrips through TOON, writes to tmp, and compares to cached original
    use std::fs;
    use std::path::Path;
    
    let tmp_dir = Path::new("/tmp/toonify_test");
    
    // Clean tmp before test
    if tmp_dir.exists() {
        fs::remove_dir_all(tmp_dir).expect("Failed to clean tmp dir before test");
    }
    fs::create_dir_all(tmp_dir).expect("Failed to create tmp dir");
    
    // Create moka cache
    let cache: Cache<String, String> = Cache::builder()
        .max_capacity(100)
        .time_to_live(Duration::from_secs(300))
        .build();
    
    // Read actual file and cache it
    let original_content = fs::read_to_string("vscode-extension/package.json")
        .expect("Failed to read vscode-extension/package.json");
    
    cache.insert("original".to_string(), original_content.clone());
    println!("=== Moka Cache Roundtrip Test ===");
    println!("Cached original file content ({} bytes)", original_content.len());
    
    // Convert to TOON
    let toon = converter::json_to_toon(&original_content)
        .expect("Failed to convert JSON to TOON");
    
    // Write TOON to tmp
    let toon_path = tmp_dir.join("package.toon");
    fs::write(&toon_path, &toon).expect("Failed to write TOON to tmp");
    println!("Wrote TOON to {:?}", toon_path);
    
    // Convert back to JSON
    let back_to_json = converter::toon_to_json(&toon)
        .expect("Failed to convert TOON back to JSON");
    
    // Write result JSON to tmp
    let result_path = tmp_dir.join("package_result.json");
    fs::write(&result_path, &back_to_json).expect("Failed to write result JSON to tmp");
    println!("Wrote result JSON to {:?}", result_path);
    
    // Get cached original
    let cached_original = cache.get(&"original".to_string())
        .expect("Failed to get cached original");
    
    // Parse both as JSON Values for comparison
    let original_value: Value = serde_json::from_str(&cached_original).unwrap();
    let result_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    // Compare JSON values (semantic equality)
    assert_eq!(original_value, result_value, "Roundtrip result differs from cached original!");
    
    // Also compare the raw strings to verify key order is preserved
    // Re-serialize both with same formatting to compare
    let original_formatted = serde_json::to_string_pretty(&original_value).unwrap();
    let result_formatted = serde_json::to_string_pretty(&result_value).unwrap();
    
    println!("\n=== Key Order Comparison ===");
    println!("Original first 500 chars:\n{}", &original_formatted[..500.min(original_formatted.len())]);
    println!("\nResult first 500 chars:\n{}", &result_formatted[..500.min(result_formatted.len())]);
    
    assert_eq!(original_formatted, result_formatted, 
        "Key order not preserved! Original and result have different key ordering.");
    
    println!("\n✓ Moka cache roundtrip test passed - content AND key order preserved!");
    
    // Clean tmp after test
    fs::remove_dir_all(tmp_dir).expect("Failed to clean tmp dir after test");
    println!("Cleaned up tmp dir");
}
