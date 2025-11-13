use serde_json::Value;

#[path = "../src/toon/mod.rs"]
mod toon;

#[path = "../src/converter.rs"]
mod converter;

#[test]
fn test_values_ending_with_colon() {
    let json = r#"{
  "data": [
    {
      "id": 1,
      "value": "message:"
    },
    {
      "id": 2,
      "value": "note:"
    }
  ]
}"#;

    println!("=== Values Ending with Colon Test ===");
    
    let toon = converter::json_to_toon(json).expect("Failed to convert JSON to TOON");
    println!("TOON:\n{}\n", toon);
    
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    println!("Back to JSON:\n{}\n", back_to_json);
    
    let original: Value = serde_json::from_str(json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    assert_eq!(original, final_value, "Values ending with colon should round-trip");
    println!("✓ Values ending with colon round-trip successful\n");
}

#[test]
fn test_plain_scalar_with_colon() {
    let json = r#"{
  "status": "ok:",
  "message": "done:"
}"#;

    println!("=== Plain Scalar with Colon Test ===");
    
    let toon = converter::json_to_toon(json).expect("Failed to convert JSON to TOON");
    println!("TOON:\n{}\n", toon);
    
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    println!("Back to JSON:\n{}\n", back_to_json);
    
    let original: Value = serde_json::from_str(json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    assert_eq!(original, final_value, "Plain scalars with trailing colon should round-trip");
    println!("✓ Plain scalars round-trip successful\n");
}

#[test]
fn test_array_followed_by_scalar() {
    let json = r#"{
  "items": ["foo", "bar"],
  "status": "ok"
}"#;

    println!("=== Array Followed by Scalar Test ===");
    
    let toon = converter::json_to_toon(json).expect("Failed to convert JSON to TOON");
    println!("TOON:\n{}\n", toon);
    
    let back_to_json = converter::toon_to_json(&toon).expect("Failed to convert TOON to JSON");
    println!("Back to JSON:\n{}\n", back_to_json);
    
    let original: Value = serde_json::from_str(json).unwrap();
    let final_value: Value = serde_json::from_str(&back_to_json).unwrap();
    
    assert_eq!(original, final_value, "Array followed by scalar field should round-trip");
    println!("✓ Array followed by scalar round-trip successful\n");
}
