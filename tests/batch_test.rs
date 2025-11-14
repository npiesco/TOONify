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

#[test]
fn test_batch_parallel_processing() {
    println!("=== Batch: Parallel processing with --parallel flag ===");
    
    let test_dir = "/tmp/batch_parallel";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();
    
    // Create 20 JSON files to make parallelism worthwhile
    println!("Creating 20 JSON files...");
    for i in 1..=20 {
        fs::write(
            format!("{}/file{:02}.json", test_dir, i),
            format!(r#"{{"id":{},"data":"item{:02}","value":{}}}"#, i, i, i * 10)
        ).unwrap();
    }
    
    println!("Running parallel batch conversion...");
    
    // Run batch with --parallel flag
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--parallel"
        ])
        .output()
        .expect("Failed to execute parallel batch command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Parallel batch conversion should succeed");
    
    // Verify all 20 files were converted
    let output_dir = format!("{}/output", test_dir);
    for i in 1..=20 {
        let toon_file = format!("{}/file{:02}.toon", output_dir, i);
        assert!(Path::new(&toon_file).exists(), "file{:02}.toon should exist", i);
        
        // Verify content is valid
        let content = fs::read_to_string(&toon_file).unwrap();
        assert!(content.contains("id,data,value") || content.contains(&format!("{}", i)), 
                "TOON file should contain converted data");
    }
    
    println!("✓ All 20 files converted successfully in parallel");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

#[test]
fn test_batch_parallel_with_errors() {
    println!("=== Batch: Parallel processing handles errors gracefully ===");
    
    let test_dir = "/tmp/batch_parallel_errors";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).unwrap();
    
    // Create mix of valid and invalid files
    fs::write(format!("{}/valid1.json", test_dir), r#"{"id":1}"#).unwrap();
    fs::write(format!("{}/valid2.json", test_dir), r#"{"id":2}"#).unwrap();
    fs::write(format!("{}/invalid1.json", test_dir), "not json at all").unwrap();
    fs::write(format!("{}/valid3.json", test_dir), r#"{"id":3}"#).unwrap();
    fs::write(format!("{}/invalid2.json", test_dir), "{broken json:").unwrap();
    fs::write(format!("{}/valid4.json", test_dir), r#"{"id":4}"#).unwrap();
    
    println!("Created 6 files (4 valid, 2 invalid)");
    
    // Run batch with --parallel flag
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--parallel"
        ])
        .output()
        .expect("Failed to execute parallel batch command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Should report errors but continue processing
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined_output = format!("{}{}", stdout, stderr);
    
    // Verify that errors are reported for invalid files
    assert!(combined_output.contains("invalid1") || combined_output.contains("error") || combined_output.contains("Error") || combined_output.contains("Failed"),
            "Should report errors for invalid files");
    
    // Verify valid files were converted
    let output_dir = format!("{}/output", test_dir);
    assert!(Path::new(&format!("{}/valid1.toon", output_dir)).exists(), "valid1.toon should exist");
    assert!(Path::new(&format!("{}/valid2.toon", output_dir)).exists(), "valid2.toon should exist");
    assert!(Path::new(&format!("{}/valid3.toon", output_dir)).exists(), "valid3.toon should exist");
    assert!(Path::new(&format!("{}/valid4.toon", output_dir)).exists(), "valid4.toon should exist");
    
    // Invalid files should not produce output
    assert!(!Path::new(&format!("{}/invalid1.toon", output_dir)).exists(), "invalid1.toon should NOT exist");
    assert!(!Path::new(&format!("{}/invalid2.toon", output_dir)).exists(), "invalid2.toon should NOT exist");
    
    println!("✓ Valid files converted, invalid files skipped");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

#[test]
fn test_batch_parallel_preserves_directory_structure() {
    println!("=== Batch: Parallel processing preserves directory structure ===");
    
    let test_dir = "/tmp/batch_parallel_recursive";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(format!("{}/dir1/subdir1", test_dir)).unwrap();
    fs::create_dir_all(format!("{}/dir1/subdir2", test_dir)).unwrap();
    fs::create_dir_all(format!("{}/dir2", test_dir)).unwrap();
    
    // Create files in nested structure
    fs::write(format!("{}/root.json", test_dir), r#"{"level":"root"}"#).unwrap();
    fs::write(format!("{}/dir1/level1.json", test_dir), r#"{"level":"dir1"}"#).unwrap();
    fs::write(format!("{}/dir1/subdir1/level2a.json", test_dir), r#"{"level":"subdir1"}"#).unwrap();
    fs::write(format!("{}/dir1/subdir2/level2b.json", test_dir), r#"{"level":"subdir2"}"#).unwrap();
    fs::write(format!("{}/dir2/level1b.json", test_dir), r#"{"level":"dir2"}"#).unwrap();
    
    println!("Created 5 files in nested directory structure");
    
    // Run parallel batch with recursive flag
    let output = Command::new(get_binary_path())
        .args(&[
            "batch",
            "--input-dir", test_dir,
            "--output-dir", &format!("{}/output", test_dir),
            "--parallel",
            "--recursive"
        ])
        .output()
        .expect("Failed to execute parallel recursive batch");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Parallel recursive batch should succeed");
    
    // Verify all files converted with structure preserved
    let output_dir = format!("{}/output", test_dir);
    assert!(Path::new(&format!("{}/root.toon", output_dir)).exists(), "root.toon should exist");
    assert!(Path::new(&format!("{}/dir1/level1.toon", output_dir)).exists(), "dir1/level1.toon should exist");
    assert!(Path::new(&format!("{}/dir1/subdir1/level2a.toon", output_dir)).exists(), "dir1/subdir1/level2a.toon should exist");
    assert!(Path::new(&format!("{}/dir1/subdir2/level2b.toon", output_dir)).exists(), "dir1/subdir2/level2b.toon should exist");
    assert!(Path::new(&format!("{}/dir2/level1b.toon", output_dir)).exists(), "dir2/level1b.toon should exist");
    
    println!("✓ Directory structure preserved in parallel processing");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    
    println!("✓ Test completed\n");
}

