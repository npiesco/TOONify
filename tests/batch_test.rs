use std::process::Command;
use std::fs;
use std::path::Path;

fn get_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/toonify", manifest_dir)
}

#[test]
fn test_batch_convert_json_to_toon() {
    println!("=== Batch: Convert multiple JSON files to TOON ===");
    
    // Create test directory and files
    let test_dir = "/tmp/batch_json_to_toon";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    // Create JSON files
    let json1 = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    let json2 = r#"{"products":[{"sku":"P001","price":19.99}]}"#;
    let json3 = r#"{"orders":[{"id":100,"total":99.99}]}"#;
    
    fs::write(format!("{}/data1.json", test_dir), json1).expect("Failed to write json1");
    fs::write(format!("{}/data2.json", test_dir), json2).expect("Failed to write json2");
    fs::write(format!("{}/data3.json", test_dir), json3).expect("Failed to write json3");
    
    println!("Created 3 JSON files");
    
    // Run batch conversion
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--from", "json",
            "--to", "toon"
        ])
        .output()
        .expect("Failed to execute batch command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Batch conversion should succeed");
    
    // Verify output files exist
    let output_dir = format!("{}/output", test_dir);
    assert!(Path::new(&format!("{}/data1.toon", output_dir)).exists(), "data1.toon should exist");
    assert!(Path::new(&format!("{}/data2.toon", output_dir)).exists(), "data2.toon should exist");
    assert!(Path::new(&format!("{}/data3.toon", output_dir)).exists(), "data3.toon should exist");
    
    // Verify content
    let toon1 = fs::read_to_string(format!("{}/data1.toon", output_dir)).expect("Failed to read toon1");
    assert!(toon1.contains("users"), "TOON should contain 'users'");
    
    println!("✓ Batch JSON to TOON conversion successful");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

#[test]
fn test_batch_convert_toon_to_json() {
    println!("=== Batch: Convert multiple TOON files to JSON ===");
    
    // Create test directory and files
    let test_dir = "/tmp/batch_toon_to_json";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    // Create TOON files
    let toon1 = "users[1]{id,name}:\n1,Alice";
    let toon2 = "products[1]{sku,price}:\nP001,19.99";
    let toon3 = "orders[1]{id,total}:\n100,99.99";
    
    fs::write(format!("{}/data1.toon", test_dir), toon1).expect("Failed to write toon1");
    fs::write(format!("{}/data2.toon", test_dir), toon2).expect("Failed to write toon2");
    fs::write(format!("{}/data3.toon", test_dir), toon3).expect("Failed to write toon3");
    
    println!("Created 3 TOON files");
    
    // Run batch conversion
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--from", "toon",
            "--to", "json"
        ])
        .output()
        .expect("Failed to execute batch command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Batch conversion should succeed");
    
    // Verify output files exist
    let output_dir = format!("{}/output", test_dir);
    assert!(Path::new(&format!("{}/data1.json", output_dir)).exists(), "data1.json should exist");
    assert!(Path::new(&format!("{}/data2.json", output_dir)).exists(), "data2.json should exist");
    assert!(Path::new(&format!("{}/data3.json", output_dir)).exists(), "data3.json should exist");
    
    // Verify content
    let json1 = fs::read_to_string(format!("{}/data1.json", output_dir)).expect("Failed to read json1");
    assert!(json1.contains("users"), "JSON should contain 'users'");
    assert!(json1.contains("Alice"), "JSON should contain 'Alice'");
    
    println!("✓ Batch TOON to JSON conversion successful");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

#[test]
fn test_batch_with_pattern_filter() {
    println!("=== Batch: Convert with pattern filter ===");
    
    let test_dir = "/tmp/batch_pattern";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    // Create mixed files
    fs::write(format!("{}/user_data.json", test_dir), r#"{"users":[{"id":1}]}"#).unwrap();
    fs::write(format!("{}/product_data.json", test_dir), r#"{"products":[{"id":2}]}"#).unwrap();
    fs::write(format!("{}/other.txt", test_dir), "ignored").unwrap();
    
    println!("Created 3 files (2 JSON, 1 txt)");
    
    // Run batch with pattern
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--pattern", "*_data.json"
        ])
        .output()
        .expect("Failed to execute batch command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Batch with pattern should succeed");
    
    // Verify only matching files were converted
    let output_dir = format!("{}/output", test_dir);
    assert!(Path::new(&format!("{}/user_data.toon", output_dir)).exists(), "user_data.toon should exist");
    assert!(Path::new(&format!("{}/product_data.toon", output_dir)).exists(), "product_data.toon should exist");
    assert!(!Path::new(&format!("{}/other.toon", output_dir)).exists(), "other.toon should NOT exist");
    
    println!("✓ Pattern filter worked correctly");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

#[test]
fn test_batch_recursive() {
    println!("=== Batch: Recursive directory conversion ===");
    
    let test_dir = "/tmp/batch_recursive";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(format!("{}/subdir1", test_dir)).unwrap();
    fs::create_dir_all(format!("{}/subdir2", test_dir)).unwrap();
    
    // Create files in different directories
    fs::write(format!("{}/root.json", test_dir), r#"{"root":true}"#).unwrap();
    fs::write(format!("{}/subdir1/sub1.json", test_dir), r#"{"sub1":true}"#).unwrap();
    fs::write(format!("{}/subdir2/sub2.json", test_dir), r#"{"sub2":true}"#).unwrap();
    
    println!("Created 3 JSON files in nested directories");
    
    // Run batch with recursive flag
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--recursive"
        ])
        .output()
        .expect("Failed to execute batch command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Recursive batch should succeed");
    
    // Verify all files were converted with directory structure preserved
    let output_dir = format!("{}/output", test_dir);
    assert!(Path::new(&format!("{}/root.toon", output_dir)).exists(), "root.toon should exist");
    assert!(Path::new(&format!("{}/subdir1/sub1.toon", output_dir)).exists(), "subdir1/sub1.toon should exist");
    assert!(Path::new(&format!("{}/subdir2/sub2.toon", output_dir)).exists(), "subdir2/sub2.toon should exist");
    
    println!("✓ Recursive conversion successful");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

#[test]
fn test_batch_reports_stats() {
    println!("=== Batch: Conversion statistics ===");
    
    let test_dir = "/tmp/batch_stats";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();
    
    // Create 5 files
    for i in 1..=5 {
        fs::write(
            format!("{}/file{}.json", test_dir, i),
            format!(r#"{{"id":{}}}"#, i)
        ).unwrap();
    }
    
    println!("Created 5 JSON files");
    
    // Run batch
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir)
        ])
        .output()
        .expect("Failed to execute batch command");
    
    println!("Exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Stdout: {}", stdout);
    println!("Stderr: {}", stderr);
    
    assert!(output.status.success(), "Batch should succeed");
    
    // Check that stats are reported
    let output_text = format!("{}{}", stdout, stderr);
    assert!(output_text.contains("5") || output_text.contains("Processed"), 
            "Should report number of files processed");
    assert!(output_text.contains("successful") || output_text.contains("completed") || output_text.contains("✓"),
            "Should report success");
    
    println!("✓ Statistics reported correctly");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

