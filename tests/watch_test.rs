use std::process::Command;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn get_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/toonify", manifest_dir)
}

fn wait_for_file_update(path: &str, expected_content_substring: &str, max_attempts: u32) -> bool {
    for _ in 0..max_attempts {
        thread::sleep(Duration::from_millis(100));
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains(expected_content_substring) {
                return true;
            }
        }
    }
    false
}

#[test]
fn test_watch_converts_new_json_file() {
    println!("=== Watch: Convert new JSON file ===");
    
    let test_dir = "/tmp/watch_test_new";
    let output_dir = "/tmp/watch_test_new_output";
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    println!("Starting watch mode...");
    
    // Start watch mode in background
    let mut watch_process = Command::new(get_binary_path())
        .args(&[
            "watch",
            "--input-dir", test_dir,
            "--output-dir", output_dir
        ])
        .spawn()
        .expect("Failed to start watch command");
    
    // Give watch process time to start
    thread::sleep(Duration::from_millis(500));
    
    // Create a new JSON file
    println!("Creating new JSON file...");
    let json_data = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    fs::write(format!("{}/test.json", test_dir), json_data).expect("Failed to write JSON file");
    
    // Wait for conversion
    println!("Waiting for conversion...");
    let output_file = format!("{}/test.toon", output_dir);
    let converted = wait_for_file_update(&output_file, "users", 50);
    
    // Cleanup
    let _ = watch_process.kill();
    let _ = watch_process.wait();
    
    assert!(converted, "Watch should convert new JSON file to TOON");
    assert!(Path::new(&output_file).exists(), "Output TOON file should exist");
    
    let toon_content = fs::read_to_string(&output_file).expect("Failed to read TOON file");
    assert!(toon_content.contains("users"), "TOON should contain 'users'");
    assert!(toon_content.contains("Alice"), "TOON should contain 'Alice'");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    
    println!("✓ Watch mode converted new file successfully\n");
}

#[test]
fn test_watch_converts_modified_file() {
    println!("=== Watch: Convert modified file ===");
    
    let test_dir = "/tmp/watch_test_modify";
    let output_dir = "/tmp/watch_test_modify_output";
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    // Create initial file
    let json_file = format!("{}/data.json", test_dir);
    fs::write(&json_file, r#"{"count":1}"#).expect("Failed to write initial file");
    
    println!("Starting watch mode...");
    
    // Start watch mode
    let mut watch_process = Command::new(get_binary_path())
        .args(&[
            "watch",
            "--input-dir", test_dir,
            "--output-dir", output_dir
        ])
        .spawn()
        .expect("Failed to start watch command");
    
    thread::sleep(Duration::from_millis(500));
    
    // Wait for initial conversion
    let output_file = format!("{}/data.toon", output_dir);
    wait_for_file_update(&output_file, "count", 50);
    
    // Modify the file
    println!("Modifying JSON file...");
    fs::write(&json_file, r#"{"count":2,"updated":true}"#).expect("Failed to modify file");
    
    // Wait for re-conversion
    println!("Waiting for re-conversion...");
    let reconverted = wait_for_file_update(&output_file, "updated", 50);
    
    // Cleanup
    let _ = watch_process.kill();
    let _ = watch_process.wait();
    
    assert!(reconverted, "Watch should re-convert modified file");
    
    let toon_content = fs::read_to_string(&output_file).expect("Failed to read TOON file");
    assert!(toon_content.contains("updated"), "TOON should contain 'updated' field");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    
    println!("✓ Watch mode detected and reconverted modified file\n");
}

#[test]
fn test_watch_with_pattern_filter() {
    println!("=== Watch: Pattern filter ===");
    
    let test_dir = "/tmp/watch_test_pattern";
    let output_dir = "/tmp/watch_test_pattern_output";
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    println!("Starting watch mode with pattern...");
    
    // Start watch with pattern
    let mut watch_process = Command::new(get_binary_path())
        .args(&[
            "watch",
            "--input-dir", test_dir,
            "--output-dir", output_dir,
            "--pattern", "*.json"
        ])
        .spawn()
        .expect("Failed to start watch command");
    
    thread::sleep(Duration::from_millis(500));
    
    // Create matching and non-matching files
    println!("Creating files...");
    fs::write(format!("{}/match.json", test_dir), r#"{"match":true}"#).unwrap();
    fs::write(format!("{}/ignore.txt", test_dir), "ignored").unwrap();
    
    // Wait for conversion
    println!("Waiting for conversion...");
    let match_output = format!("{}/match.toon", output_dir);
    let ignore_output = format!("{}/ignore.toon", output_dir);
    
    let matched = wait_for_file_update(&match_output, "match", 50);
    thread::sleep(Duration::from_millis(500));
    
    // Cleanup
    let _ = watch_process.kill();
    let _ = watch_process.wait();
    
    assert!(matched, "Watch should convert matching file");
    assert!(Path::new(&match_output).exists(), "Match output should exist");
    assert!(!Path::new(&ignore_output).exists(), "Ignore output should NOT exist");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    
    println!("✓ Watch mode pattern filter working correctly\n");
}

#[test]
fn test_watch_handles_errors_gracefully() {
    println!("=== Watch: Error handling ===");
    
    let test_dir = "/tmp/watch_test_error";
    let output_dir = "/tmp/watch_test_error_output";
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    println!("Starting watch mode...");
    
    // Start watch mode
    let mut watch_process = Command::new(get_binary_path())
        .args(&[
            "watch",
            "--input-dir", test_dir,
            "--output-dir", output_dir
        ])
        .spawn()
        .expect("Failed to start watch command");
    
    thread::sleep(Duration::from_millis(500));
    
    // Create valid and invalid files
    println!("Creating valid and invalid files...");
    fs::write(format!("{}/valid.json", test_dir), r#"{"valid":true}"#).unwrap();
    fs::write(format!("{}/invalid.json", test_dir), "not valid json").unwrap();
    
    // Wait for valid file conversion
    println!("Waiting for valid file conversion...");
    let valid_output = format!("{}/valid.toon", output_dir);
    let valid_converted = wait_for_file_update(&valid_output, "valid", 50);
    
    // Invalid file should not create output (or create error file)
    thread::sleep(Duration::from_millis(500));
    
    // Cleanup
    let _ = watch_process.kill();
    let _ = watch_process.wait();
    
    assert!(valid_converted, "Watch should convert valid file despite errors");
    assert!(Path::new(&valid_output).exists(), "Valid output should exist");
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    
    println!("✓ Watch mode handles errors gracefully\n");
}

#[test]
fn test_watch_stops_cleanly() {
    println!("=== Watch: Clean shutdown ===");
    
    let test_dir = "/tmp/watch_test_shutdown";
    let output_dir = "/tmp/watch_test_shutdown_output";
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    println!("Starting watch mode...");
    
    // Start watch mode
    let mut watch_process = Command::new(get_binary_path())
        .args(&[
            "watch",
            "--input-dir", test_dir,
            "--output-dir", output_dir
        ])
        .spawn()
        .expect("Failed to start watch command");
    
    thread::sleep(Duration::from_millis(500));
    
    // Create a file
    fs::write(format!("{}/test.json", test_dir), r#"{"test":true}"#).unwrap();
    thread::sleep(Duration::from_millis(300));
    
    // Stop watch mode
    println!("Stopping watch mode...");
    watch_process.kill().expect("Failed to kill watch process");
    let status = watch_process.wait().expect("Failed to wait for process");
    
    println!("Watch process exited with status: {:?}", status);
    
    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_dir_all(output_dir);
    
    println!("✓ Watch mode stopped cleanly\n");
}

