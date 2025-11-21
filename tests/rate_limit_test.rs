use std::process::{Command, Child, Stdio};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;

// Global mutex to ensure server tests run serially (avoid port conflicts)
static SERVER_TEST_LOCK: Mutex<()> = Mutex::new(());

fn cleanup_servers() {
    // Kill any existing toonify processes
    let _ = Command::new("pkill")
        .args(&["-9", "toonify"])
        .output();
    thread::sleep(Duration::from_millis(500));
}

fn start_server_with_rate_limit(rate_limit: u32, window: u64) -> Child {
    // Use the pre-built binary directly instead of cargo run
    let binary_path = "./target/release/toonify";
    
    println!("Starting server: {} serve --rate-limit {} --rate-limit-window {}", 
             binary_path, rate_limit, window);
    
    let child = Command::new(binary_path)
        .args(&[
            "serve",
            "--rate-limit",
            &rate_limit.to_string(),
            "--rate-limit-window",
            &window.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Give server time to start
    thread::sleep(Duration::from_millis(1500));
    
    child
}

fn wait_for_server() {
    // Wait for server to be ready
    for _ in 0..30 {
        if let Ok(response) = reqwest::blocking::get("http://localhost:5000/") {
            if response.status().is_success() {
                return;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("Server did not start in time");
}

#[test]
fn test_rate_limit_enforced() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    cleanup_servers();
    
    println!("=== Rate Limit Test: Enforce 5 requests per second ===");
    
    // Start server with strict rate limit: 5 requests per second
    let mut server = start_server_with_rate_limit(5, 1);
    wait_for_server();
    
    let client = reqwest::blocking::Client::new();
    let url = "http://localhost:5000/json-to-toon";
    let payload = serde_json::json!({
        "data": r#"{"test": "data"}"#
    });
    
    // Make 5 requests - should all succeed
    println!("Making 5 requests (should all succeed)...");
    for i in 1..=5 {
        let response = client
            .post(url)
            .json(&payload)
            .send()
            .expect("Failed to send request");
        
        println!("  Request {}: Status {}", i, response.status());
        assert_eq!(
            response.status(),
            reqwest::StatusCode::OK,
            "Request {} should succeed (within rate limit)",
            i
        );
    }
    
    // 6th request should be rate limited
    println!("Making 6th request (should be rate limited)...");
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .expect("Failed to send request");
    
    println!("  Request 6: Status {}", response.status());
    assert_eq!(
        response.status(),
        reqwest::StatusCode::TOO_MANY_REQUESTS,
        "6th request should be rate limited (429)"
    );
    
    // Wait for rate limit window to reset
    println!("Waiting 1.5 seconds for rate limit to reset...");
    thread::sleep(Duration::from_millis(1500));
    
    // After reset, request should succeed again
    println!("Making request after reset (should succeed)...");
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .expect("Failed to send request");
    
    println!("  Post-reset request: Status {}", response.status());
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "Request after reset should succeed"
    );
    
    server.kill().expect("Failed to kill server");
    println!("=== Rate Limit Test: PASSED ===");
}

#[test]
fn test_rate_limit_with_burst() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    cleanup_servers();
    
    println!("=== Rate Limit Test: Burst handling (10 requests per 2 seconds) ===");
    
    // Start server with burst capacity: 10 requests per 2 seconds
    let mut server = start_server_with_rate_limit(10, 2);
    wait_for_server();
    
    let client = reqwest::blocking::Client::new();
    let url = "http://localhost:5000/json-to-toon";
    let payload = serde_json::json!({
        "data": r#"{"burst": "test"}"#
    });
    
    // Rapidly make 10 requests - burst capacity is 10, so 10 should succeed
    println!("Making 10 rapid requests (burst, should all succeed)...");
    for i in 1..=10 {
        let response = client
            .post(url)
            .json(&payload)
            .send()
            .expect("Failed to send request");
        
        println!("  Request {}: Status {}", i, response.status());
        // With burst_size(10) and minimal/no delay, we expect 10 successes
        if i <= 10 {
            assert_eq!(
                response.status(),
                reqwest::StatusCode::OK,
                "Request {} should succeed (within burst of 10)",
                i
            );
        }
    }
    
    // Immediately make 11th request - should be rate limited as burst is exhausted
    println!("Making 11th request immediately (should be rate limited)...");
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .expect("Failed to send request");
    
    println!("  Request 11: Status {}", response.status());
    assert_eq!(
        response.status(),
        reqwest::StatusCode::TOO_MANY_REQUESTS,
        "11th request should be rate limited (burst exhausted)"
    );
    
    server.kill().expect("Failed to kill server");
    println!("=== Rate Limit Test: PASSED ===");
}

#[test]
fn test_rate_limit_disabled_by_default() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    cleanup_servers();
    
    println!("=== Rate Limit Test: Disabled by default ===");
    
    // Start server WITHOUT rate limiting flags
    let binary_path = "./target/release/toonify";
    println!("Starting server without rate limiting: {} serve", binary_path);
    
    let mut server = Command::new(binary_path)
        .args(&["serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");
    
    wait_for_server();
    
    let client = reqwest::blocking::Client::new();
    let url = "http://localhost:5000/json-to-toon";
    let payload = serde_json::json!({
        "data": r#"{"unlimited": "requests"}"#
    });
    
    // Make many requests rapidly - none should be rate limited
    println!("Making 20 rapid requests without rate limiting...");
    for i in 1..=20 {
        let response = client
            .post(url)
            .json(&payload)
            .send()
            .expect("Failed to send request");
        
        if i % 5 == 0 {
            println!("  Request {}: Status {}", i, response.status());
        }
        
        assert_eq!(
            response.status(),
            reqwest::StatusCode::OK,
            "Request {} should succeed (no rate limiting enabled)",
            i
        );
    }
    
    server.kill().expect("Failed to kill server");
    println!("=== Rate Limit Test: PASSED ===");
}

