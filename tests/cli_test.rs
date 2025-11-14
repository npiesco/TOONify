use std::fs;
use std::process::Command;
use std::path::PathBuf;
use serde_json::Value;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("toonify");
    path
}

fn create_temp_file(name: &str, content: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("tmp");
    fs::create_dir_all(&path).expect("Failed to create tmp dir");
    path.push(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

fn cleanup_temp_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_cli_convert_json_file_to_toon() {
    println!("=== CLI: Convert JSON File to TOON ===");
    
    let json_content = r#"{
  "users": [
    {
      "id": 1,
      "name": "Alice",
      "email": "alice@example.com"
    },
    {
      "id": 2,
      "name": "Bob",
      "email": "bob@example.com"
    }
  ]
}"#;
    
    let input_file = create_temp_file("test_input.json", json_content);
    println!("Created temp file: {:?}", input_file);
    println!("Input JSON:\n{}\n", json_content);
    
    let binary = get_binary_path();
    println!("Running: {:?} convert {:?}", binary, input_file);
    
    let output = Command::new(&binary)
        .arg("convert")
        .arg(&input_file)
        .output()
        .expect("Failed to execute toonify binary");
    
    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "CLI command should succeed");
    
    let toon_output = String::from_utf8_lossy(&output.stdout);
    assert!(!toon_output.is_empty(), "Should produce TOON output");
    assert!(toon_output.contains("users"), "TOON should contain 'users' key");
    assert!(toon_output.contains("Alice"), "TOON should contain data");
    
    println!("✓ CLI convert command successful\n");
    
    cleanup_temp_file(&input_file);
}

#[test]
fn test_cli_convert_json_to_output_file() {
    println!("=== CLI: Convert JSON to Output File ===");
    
    let json_content = r#"{"status": "ok", "count": 42}"#;
    
    let input_file = create_temp_file("test_input2.json", json_content);
    let output_file = create_temp_file("test_output.toon", "");
    let _ = fs::remove_file(&output_file); // Remove it so CLI creates it
    
    println!("Input: {:?}", input_file);
    println!("Output: {:?}", output_file);
    
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("convert")
        .arg(&input_file)
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to execute toonify binary");
    
    println!("Exit status: {}", output.status);
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "CLI command should succeed");
    assert!(output_file.exists(), "Output file should be created");
    
    let toon_content = fs::read_to_string(&output_file).expect("Failed to read output file");
    println!("Output TOON file content:\n{}\n", toon_content);
    
    assert!(!toon_content.is_empty(), "Output file should not be empty");
    assert!(toon_content.contains("status"), "TOON should contain data");
    
    println!("✓ CLI convert with output file successful\n");
    
    cleanup_temp_file(&input_file);
    cleanup_temp_file(&output_file);
}

#[test]
fn test_cli_convert_toon_file_to_json() {
    println!("=== CLI: Convert TOON File to JSON ===");
    
    let toon_content = r#"products[2]{id,name,price}:
1,Laptop,999.99
2,Mouse,29.99"#;
    
    let input_file = create_temp_file("test_input.toon", toon_content);
    println!("Created TOON file: {:?}", input_file);
    println!("Input TOON:\n{}\n", toon_content);
    
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("convert")
        .arg(&input_file)
        .output()
        .expect("Failed to execute toonify binary");
    
    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "CLI command should succeed");
    
    let json_output = String::from_utf8_lossy(&output.stdout);
    assert!(!json_output.is_empty(), "Should produce JSON output");
    
    let parsed: Value = serde_json::from_str(&json_output.to_string())
        .expect("Output should be valid JSON");
    
    assert!(parsed.get("products").is_some(), "JSON should have products array");
    
    println!("✓ CLI TOON to JSON conversion successful\n");
    
    cleanup_temp_file(&input_file);
}

#[test]
fn test_cli_help_command() {
    println!("=== CLI: Help Command ===");
    
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute toonify binary");
    
    println!("Exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Help output:\n{}\n", stdout);
    
    assert!(output.status.success(), "Help command should succeed");
    assert!(stdout.contains("toonify") || stdout.contains("convert"), 
            "Help should mention convert command");
    
    println!("✓ CLI help command successful\n");
}

#[test]
fn test_cli_stdin_to_stdout() {
    println!("=== CLI: STDIN to STDOUT Conversion ===");
    
    let json_input = r#"{"message":"hello","value":123}"#;
    
    let binary = get_binary_path();
    let mut child = Command::new(&binary)
        .arg("convert")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn toonify binary");
    
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json_input.as_bytes()).expect("Failed to write to stdin");
    }
    
    let output = child.wait_with_output().expect("Failed to wait for child");
    
    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "STDIN conversion should succeed");
    
    let toon_output = String::from_utf8_lossy(&output.stdout);
    assert!(!toon_output.is_empty(), "Should produce TOON output");
    assert!(toon_output.contains("message"), "TOON should contain data");
    
    println!("✓ CLI STDIN to STDOUT conversion successful\n");
}

