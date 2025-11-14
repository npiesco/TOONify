use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::net::TcpListener;

fn is_port_free(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

fn wait_for_server_ready(max_attempts: u32) -> bool {
    for attempt in 0..max_attempts {
        let result = Command::new("curl")
            .arg("-s")
            .arg("http://localhost:5000/")
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.contains("TOONify") {
                    println!("Server ready after {} attempts", attempt + 1);
                    return true;
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn test_streaming_json_to_toon() {
    println!("=== Streaming: JSON to TOON via HTTP ===");
    
    // Skip if port is already in use
    if !is_port_free(5000) {
        println!("Skipping test: Port 5000 already in use");
        return;
    }
    
    // Start server in background
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server to be ready by checking health endpoint
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // Test streaming conversion
    let json_data = r#"{"message":"hello","value":123}"#;
    
    let output = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output();
    
    cleanup();
    
    let output = output.expect("Failed to execute curl");
    println!("Status: {}", output.status);
    println!("Response: {}", String::from_utf8_lossy(&output.stdout));
    
    assert!(output.status.success(), "HTTP request should succeed");
    
    let response = String::from_utf8_lossy(&output.stdout);
    assert!(response.contains("result") || response.contains("message"), 
            "Response should contain conversion result");
    
    println!("✓ Streaming JSON to TOON successful\n");
}

#[test]
fn test_streaming_toon_to_json() {
    println!("=== Streaming: TOON to JSON via HTTP ===");
    
    // Skip if port is already in use
    if !is_port_free(5000) {
        println!("Skipping test: Port 5000 already in use");
        return;
    }
    
    // Start server in background
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server to be ready by checking health endpoint
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // Test streaming conversion
    let toon_data = "message:hello\\n\\nvalue:123";
    
    let output = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/toon-to-json",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, toon_data)
        ])
        .output();
    
    cleanup();
    
    let output = output.expect("Failed to execute curl");
    println!("Status: {}", output.status);
    println!("Response: {}", String::from_utf8_lossy(&output.stdout));
    
    assert!(output.status.success(), "HTTP request should succeed");
    
    let response = String::from_utf8_lossy(&output.stdout);
    assert!(response.contains("result") || response.contains("message"), 
            "Response should contain conversion result");
    
    println!("✓ Streaming TOON to JSON successful\n");
}

#[test]
fn test_streaming_large_payload() {
    println!("=== Streaming: Large Payload ===");
    
    // Skip if port is already in use
    if !is_port_free(5000) {
        println!("Skipping test: Port 5000 already in use");
        return;
    }
    
    // Start server in background
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server to be ready by checking health endpoint
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // Create large JSON payload (1000 users)
    let users: Vec<String> = (0..1000)
        .map(|i| format!(r#"{{"id":{},"name":"User{}","email":"user{}@example.com"}}"#, i, i, i))
        .collect();
    
    let large_json = format!(r#"{{"users":[{}]}}"#, users.join(","));
    println!("Large payload size: {} bytes", large_json.len());
    
    // Write to temp file
    let temp_file = "/tmp/large_payload.json";
    std::fs::write(temp_file, &large_json).expect("Failed to write temp file");
    
    let json_data = format!(r#"{{"data":"{}"}}"#, large_json.replace("\"", "\\\""));
    std::fs::write("/tmp/curl_data.json", &json_data).expect("Failed to write curl data");
    
    let output = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!("@/tmp/curl_data.json")
        ])
        .output();
    
    cleanup();
    
    // Cleanup temp files
    let _ = std::fs::remove_file(temp_file);
    let _ = std::fs::remove_file("/tmp/curl_data.json");
    
    let output = output.expect("Failed to execute curl");
    println!("Status: {}", output.status);
    
    let response = String::from_utf8_lossy(&output.stdout);
    let response_size = response.len();
    println!("Response size: {} bytes", response_size);
    
    assert!(output.status.success(), "HTTP request should succeed");
    assert!(response_size > 0, "Should have response");
    
    println!("✓ Streaming large payload successful\n");
}

#[test]
fn test_streaming_concurrent_requests() {
    println!("=== Streaming: Concurrent Requests ===");
    
    // Skip if port is already in use
    if !is_port_free(5000) {
        println!("Skipping test: Port 5000 already in use");
        return;
    }
    
    // Start server in background
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server to be ready by checking health endpoint
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // Spawn multiple concurrent requests
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let json_data = format!(r#"{{"id":{},"message":"request{}"}}"#, i, i);
                
                Command::new("curl")
                    .args(&[
                        "-X", "POST",
                        "http://localhost:5000/json-to-toon",
                        "-H", "Content-Type: application/json",
                        "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
                    ])
                    .output()
                    .expect("Failed to execute curl")
            })
        })
        .collect();
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();
    
    cleanup();
    
    let successful = results.iter().filter(|r| r.status.success()).count();
    println!("Successful requests: {}/10", successful);
    
    assert!(successful >= 8, "At least 8/10 requests should succeed (got {})", successful);
    
    println!("✓ Concurrent requests handled successfully\n");
}

#[test]
fn test_streaming_health_check() {
    println!("=== Streaming: Health Check ===");
    
    // Skip if port is already in use
    if !is_port_free(5000) {
        println!("Skipping test: Port 5000 already in use");
        return;
    }
    
    // Start server in background
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server to be ready by checking health endpoint
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
    };
    
    let output = Command::new("curl")
        .arg("http://localhost:5000/")
        .output();
    
    cleanup();
    
    let output = output.expect("Failed to execute curl");
    println!("Status: {}", output.status);
    println!("Response: {}", String::from_utf8_lossy(&output.stdout));
    
    assert!(output.status.success(), "Health check should succeed");
    
    let response = String::from_utf8_lossy(&output.stdout);
    assert!(response.contains("TOONify"), "Health check should mention TOONify");
    
    println!("✓ Health check successful\n");
}

