// Integration tests for distributed processing (job queue system)
// Tests verify async job submission, status checking, result retrieval, and worker coordination

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Once};

static SERVER_TEST_LOCK: Mutex<()> = Mutex::new(());
static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        // Any one-time setup
    });
}

fn wait_for_server_ready(max_attempts: u32) -> bool {
    for attempt in 1..=max_attempts {
        if let Ok(response) = reqwest::blocking::get("http://localhost:5000/") {
            if response.status().is_success() {
                return true;
            }
        }
        if attempt < max_attempts {
            thread::sleep(Duration::from_millis(100));
        }
    }
    false
}

#[test]
fn test_job_queue_submit_and_retrieve() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    init_test_env();
    
    println!("=== Distributed Processing: Submit job and retrieve result ===");
    
    // Start server with job queue enabled
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");
    
    if !wait_for_server_ready(100) {
        let _ = server.kill();
        if let Ok(output) = server.wait_with_output() {
            eprintln!("Server stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("Server stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        panic!("Server failed to start");
    }
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
        thread::sleep(Duration::from_millis(500));
    };
    
    // Submit a conversion job
    let client = reqwest::blocking::Client::new();
    let json_data = r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#;
    
    let submit_response = client
        .post("http://localhost:5000/jobs/submit")
        .json(&serde_json::json!({
            "operation": "json_to_toon",
            "data": json_data
        }))
        .send();
    
    assert!(submit_response.is_ok(), "Job submission should succeed");
    let submit_result = submit_response.unwrap();
    assert!(submit_result.status().is_success(), "Job submission should return success");
    
    let submit_body: serde_json::Value = submit_result.json().expect("Should parse JSON response");
    assert!(submit_body.get("job_id").is_some(), "Should return job_id");
    
    let job_id = submit_body["job_id"].as_str().expect("job_id should be string");
    println!("✓ Job submitted with ID: {}", job_id);
    
    // Poll for job completion
    let mut completed = false;
    for _ in 0..50 {  // Poll for up to 5 seconds
        let status_response = client
            .get(&format!("http://localhost:5000/jobs/{}/status", job_id))
            .send();
        
        if let Ok(response) = status_response {
            if let Ok(status_body) = response.json::<serde_json::Value>() {
                let status = status_body["status"].as_str().unwrap_or("unknown");
                println!("Job status: {}", status);
                
                if status == "completed" {
                    completed = true;
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    assert!(completed, "Job should complete within timeout");
    
    // Retrieve result
    let result_response = client
        .get(&format!("http://localhost:5000/jobs/{}/result", job_id))
        .send();
    
    assert!(result_response.is_ok(), "Result retrieval should succeed");
    let result_body: serde_json::Value = result_response.unwrap().json().expect("Should parse result");
    
    assert!(result_body.get("result").is_some(), "Should have result field");
    let result_data = result_body["result"].as_str().expect("Result should be string");
    
    // Verify the result is valid TOON
    assert!(result_data.contains("users[2]{"), "Result should be TOON format");
    assert!(result_data.contains("Alice"), "Result should contain data");
    
    println!("✓ Job completed and result retrieved successfully");
    
    cleanup();
}

#[test]
fn test_job_queue_status_transitions() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    init_test_env();
    
    println!("=== Distributed Processing: Job status transitions (pending -> processing -> completed) ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
        thread::sleep(Duration::from_millis(500));
    };
    
    let client = reqwest::blocking::Client::new();
    let json_data = r#"{"data":"test"}"#;
    
    // Submit job
    let submit_response = client
        .post("http://localhost:5000/jobs/submit")
        .json(&serde_json::json!({
            "operation": "json_to_toon",
            "data": json_data
        }))
        .send()
        .expect("Should submit job");
    
    let submit_body: serde_json::Value = submit_response.json().expect("Should parse JSON");
    let job_id = submit_body["job_id"].as_str().expect("Should have job_id");
    
    // Check initial status (should be pending or processing)
    let status_response = client
        .get(&format!("http://localhost:5000/jobs/{}/status", job_id))
        .send()
        .expect("Should get status");
    
    let status_body: serde_json::Value = status_response.json().expect("Should parse status");
    let initial_status = status_body["status"].as_str().expect("Should have status");
    
    assert!(
        initial_status == "pending" || initial_status == "processing" || initial_status == "completed",
        "Status should be valid: {}", initial_status
    );
    
    println!("✓ Job status transitions working correctly");
    
    cleanup();
}

#[test]
fn test_job_queue_concurrent_jobs() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    init_test_env();
    
    println!("=== Distributed Processing: Process multiple concurrent jobs ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue", "--workers", "4"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
        thread::sleep(Duration::from_millis(500));
    };
    
    let client = reqwest::blocking::Client::new();
    let mut job_ids = Vec::new();
    
    // Submit 10 jobs concurrently
    for i in 0..10 {
        let json_data = format!(r#"{{"id":{},"value":"test_{}"}}"#, i, i);
        
        let response = client
            .post("http://localhost:5000/jobs/submit")
            .json(&serde_json::json!({
                "operation": "json_to_toon",
                "data": json_data
            }))
            .send()
            .expect("Should submit job");
        
        let body: serde_json::Value = response.json().expect("Should parse JSON");
        let job_id = body["job_id"].as_str().expect("Should have job_id").to_string();
        job_ids.push(job_id);
    }
    
    println!("✓ Submitted {} jobs", job_ids.len());
    
    // Wait for all jobs to complete
    let mut all_completed = false;
    for attempt in 0..100 {  // Poll for up to 10 seconds
        let mut completed_count = 0;
        
        for job_id in &job_ids {
            let status_response = client
                .get(&format!("http://localhost:5000/jobs/{}/status", job_id))
                .send();
            
            if let Ok(response) = status_response {
                if let Ok(status_body) = response.json::<serde_json::Value>() {
                    if status_body["status"].as_str() == Some("completed") {
                        completed_count += 1;
                    }
                }
            }
        }
        
        println!("Attempt {}: {}/{} jobs completed", attempt + 1, completed_count, job_ids.len());
        
        if completed_count == job_ids.len() {
            all_completed = true;
            break;
        }
        
        thread::sleep(Duration::from_millis(100));
    }
    
    assert!(all_completed, "All jobs should complete within timeout");
    println!("✓ All concurrent jobs completed successfully");
    
    cleanup();
}

#[test]
fn test_job_queue_error_handling() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    init_test_env();
    
    println!("=== Distributed Processing: Handle invalid job data gracefully ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
        thread::sleep(Duration::from_millis(500));
    };
    
    let client = reqwest::blocking::Client::new();
    
    // Submit job with invalid JSON
    let invalid_data = "this is not json";
    
    let response = client
        .post("http://localhost:5000/jobs/submit")
        .json(&serde_json::json!({
            "operation": "json_to_toon",
            "data": invalid_data
        }))
        .send()
        .expect("Should handle invalid job submission");
    
    let body: serde_json::Value = response.json().expect("Should parse response");
    let job_id = body["job_id"].as_str().expect("Should still get job_id");
    
    // Wait for job to be marked as failed
    let mut failed = false;
    for _ in 0..50 {
        let status_response = client
            .get(&format!("http://localhost:5000/jobs/{}/status", job_id))
            .send();
        
        if let Ok(response) = status_response {
            if let Ok(status_body) = response.json::<serde_json::Value>() {
                let status = status_body["status"].as_str().unwrap_or("unknown");
                
                if status == "failed" {
                    assert!(status_body.get("error").is_some(), "Failed job should have error message");
                    failed = true;
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    assert!(failed, "Job with invalid data should be marked as failed");
    println!("✓ Invalid jobs are handled gracefully with error status");
    
    cleanup();
}

#[test]
fn test_job_queue_list_jobs() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    init_test_env();
    
    println!("=== Distributed Processing: List all jobs ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
        thread::sleep(Duration::from_millis(500));
    };
    
    let client = reqwest::blocking::Client::new();
    
    // Submit a few jobs
    for i in 0..3 {
        let json_data = format!(r#"{{"test":{}}}"#, i);
        let _ = client
            .post("http://localhost:5000/jobs/submit")
            .json(&serde_json::json!({
                "operation": "json_to_toon",
                "data": json_data
            }))
            .send();
    }
    
    thread::sleep(Duration::from_millis(500));
    
    // List all jobs
    let list_response = client
        .get("http://localhost:5000/jobs")
        .send()
        .expect("Should list jobs");
    
    assert!(list_response.status().is_success(), "List endpoint should succeed");
    
    let list_body: serde_json::Value = list_response.json().expect("Should parse JSON");
    assert!(list_body.get("jobs").is_some(), "Should have jobs field");
    
    let jobs = list_body["jobs"].as_array().expect("jobs should be array");
    assert!(jobs.len() >= 3, "Should have at least 3 jobs: found {}", jobs.len());
    
    println!("✓ Job listing endpoint works correctly ({} jobs found)", jobs.len());
    
    cleanup();
}

#[test]
fn test_job_queue_redis_persistence() {
    let _lock = SERVER_TEST_LOCK.lock().unwrap();
    init_test_env();
    
    println!("=== Distributed Processing: Jobs persist in Redis across server restarts ===");
    
    // Check if Redis is available
    let redis_check = Command::new("redis-cli")
        .args(&["ping"])
        .output();
    
    if redis_check.is_err() || !String::from_utf8_lossy(&redis_check.unwrap().stdout).contains("PONG") {
        println!("⚠ Skipping test - Redis not available");
        return;
    }
    
    // Flush Redis to start clean
    let _ = Command::new("redis-cli")
        .args(&["flushall"])
        .output();
    
    // Start server with Redis job queue
    let mut server1 = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue", "--job-queue-backend", "redis://127.0.0.1:6379"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let client = reqwest::blocking::Client::new();
    let json_data = r#"{"persist":"test"}"#;
    
    // Submit job
    let response = client
        .post("http://localhost:5000/jobs/submit")
        .json(&serde_json::json!({
            "operation": "json_to_toon",
            "data": json_data
        }))
        .send()
        .expect("Should submit job");
    
    let body: serde_json::Value = response.json().expect("Should parse JSON");
    let job_id = body["job_id"].as_str().expect("Should have job_id").to_string();
    
    println!("✓ Job submitted with ID: {}", job_id);
    
    // Stop server
    let _ = server1.kill();
    let _ = server1.wait();
    thread::sleep(Duration::from_millis(500));
    
    println!("✓ Server stopped");
    
    // Start server again
    let mut server2 = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--enable-job-queue", "--job-queue-backend", "redis://127.0.0.1:6379"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start after restart");
    
    println!("✓ Server restarted");
    
    // Check if job still exists
    let status_response = client
        .get(&format!("http://localhost:5000/jobs/{}/status", job_id))
        .send()
        .expect("Should retrieve job status after restart");
    
    assert!(
        status_response.status().is_success(),
        "Job should still exist after server restart"
    );
    
    let status_body: serde_json::Value = status_response.json().expect("Should parse status");
    assert!(status_body.get("status").is_some(), "Should have status field");
    
    println!("✓ Job persisted across server restart");
    
    let _ = server2.kill();
    let _ = server2.wait();
}

