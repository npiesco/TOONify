use serde_json::Value;

#[path = "../src/toon/mod.rs"]
mod toon;

#[path = "../src/converter.rs"]
mod converter;

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

    let original_value: Value = toon::parse_toon(original_toon).expect("Failed to parse original TOON");
    let final_value: Value = toon::parse_toon(&final_toon).expect("Failed to parse final TOON");

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
