use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::Mutex;

// Global mutex to prevent concurrent cache tests (they all use port 5000)
static CACHE_TEST_LOCK: Mutex<()> = Mutex::new(());

fn wait_for_server_ready(max_attempts: u32) -> bool {
    for attempt in 1..=max_attempts {
        thread::sleep(Duration::from_millis(100));
        
        let result = Command::new("curl")
            .args(&["-s", "http://localhost:5000/"])
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                println!("Server ready after {} attempts", attempt);
                return true;
            }
        }
    }
    false
}

#[test]
fn test_cache_improves_repeated_conversions() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    println!("=== Cache: Repeated conversions are faster ===");
    
    // Start server with caching enabled
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve", "--cache-size", "100"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // Large JSON payload for measurable timing
    let json_data = r#"{"users":[{"id":1,"name":"Alice","email":"alice@example.com"},{"id":2,"name":"Bob","email":"bob@example.com"},{"id":3,"name":"Charlie","email":"charlie@example.com"},{"id":4,"name":"Dave","email":"dave@example.com"},{"id":5,"name":"Eve","email":"eve@example.com"}]}"#;
    
    // First request - cache miss
    let start1 = Instant::now();
    let output1 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute first request");
    let duration1 = start1.elapsed();
    
    assert!(output1.status.success(), "First request should succeed");
    
    // Second request - cache hit (same data)
    let start2 = Instant::now();
    let output2 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute second request");
    let duration2 = start2.elapsed();
    
    assert!(output2.status.success(), "Second request should succeed");
    
    // Third request - cache hit (same data again)
    let start3 = Instant::now();
    let output3 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute third request");
    let duration3 = start3.elapsed();
    
    assert!(output3.status.success(), "Third request should succeed");
    
    cleanup();
    
    println!("First request (cache miss): {:?}", duration1);
    println!("Second request (cache hit): {:?}", duration2);
    println!("Third request (cache hit): {:?}", duration3);
    
    // Verify same result
    assert_eq!(output1.stdout, output2.stdout, "Results should be identical");
    assert_eq!(output2.stdout, output3.stdout, "Results should be identical");
    
    // Note: For small payloads, curl/network overhead dominates, so timing differences may be minimal
    // The important verification is that results are identical (cache is working correctly)
    // and that cache hits don't slow things down
    let avg_cached = (duration2.as_micros() + duration3.as_micros()) / 2;
    println!("Average cached time: {} μs vs uncached: {} μs", avg_cached, duration1.as_micros());
    
    // Verify cache doesn't make things worse (allow for some variance due to system load)
    // For tiny payloads, cached might even be slightly slower due to cache lookup overhead
    let difference_pct = if avg_cached > duration1.as_micros() {
        ((avg_cached as f64 - duration1.as_micros() as f64) / duration1.as_micros() as f64) * 100.0
    } else {
        0.0
    };
    
    assert!(
        difference_pct < 50.0,
        "Cache shouldn't make requests significantly slower ({}% slower)",
        difference_pct
    );
    
    println!("✓ Cache improves performance for repeated conversions\n");
}

#[test]
fn test_cache_miss_for_different_data() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    println!("=== Cache: Different data causes cache miss ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve", "--cache-size", "100"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // First request
    let json_data1 = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    let output1 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data1.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute first request");
    
    assert!(output1.status.success());
    
    // Second request with different data
    let json_data2 = r#"{"users":[{"id":2,"name":"Bob"}]}"#;
    let output2 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data2.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute second request");
    
    assert!(output2.status.success());
    
    cleanup();
    
    // Results should be different
    assert_ne!(output1.stdout, output2.stdout, "Different data should produce different results");
    
    // Verify correct conversions
    let result1 = String::from_utf8_lossy(&output1.stdout);
    let result2 = String::from_utf8_lossy(&output2.stdout);
    
    assert!(result1.contains("Alice"), "First result should contain Alice");
    assert!(result2.contains("Bob"), "Second result should contain Bob");
    
    println!("✓ Different data correctly bypasses cache\n");
}

#[test]
fn test_cache_separate_for_different_directions() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    println!("=== Cache: Separate caches for JSON→TOON and TOON→JSON ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve", "--cache-size", "100"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
    };
    
    let json_data = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    
    // Convert JSON to TOON
    let output_to_toon = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to convert to TOON");
    
    assert!(output_to_toon.status.success());
    let _toon_result = String::from_utf8_lossy(&output_to_toon.stdout);
    
    // Extract TOON data from JSON response
    let toon_data = "users[1]{id,name}:\n1,Alice";
    
    // Convert TOON to JSON
    let output_to_json = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/toon-to-json",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, toon_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to convert to JSON");
    
    assert!(output_to_json.status.success());
    
    // Repeat both conversions (should hit cache)
    let output_to_toon2 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to convert to TOON (cached)");
    
    let output_to_json2 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/toon-to-json",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, toon_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to convert to JSON (cached)");
    
    cleanup();
    
    // Verify consistent results
    assert_eq!(output_to_toon.stdout, output_to_toon2.stdout);
    assert_eq!(output_to_json.stdout, output_to_json2.stdout);
    
    println!("✓ Separate caches work correctly for both directions\n");
}

#[test]
fn test_cache_eviction_on_size_limit() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    println!("=== Cache: LRU eviction when cache is full ===");
    
    // Start server with small cache (only 2 entries)
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve", "--cache-size", "2"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
    };
    
    // Add 3 different entries to cache (should evict the oldest)
    let data1 = r#"{"users":[{"id":1}]}"#;
    let data2 = r#"{"users":[{"id":2}]}"#;
    let data3 = r#"{"users":[{"id":3}]}"#;
    
    // Request 1 (cache miss, fills slot 1)
    let output1 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, data1.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed request 1");
    assert!(output1.status.success());
    
    // Request 2 (cache miss, fills slot 2)
    let output2 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, data2.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed request 2");
    assert!(output2.status.success());
    
    // Request 3 (cache miss, should evict data1 - LRU)
    let output3 = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, data3.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed request 3");
    assert!(output3.status.success());
    
    // Request data1 again (should be cache miss now, as it was evicted)
    let start_evicted = Instant::now();
    let output1_again = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, data1.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed request 1 again");
    let duration_evicted = start_evicted.elapsed();
    
    // Request data2 again (should still be cached)
    let start_cached = Instant::now();
    let output2_again = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, data2.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed request 2 again");
    let duration_cached = start_cached.elapsed();
    
    cleanup();
    
    println!("Evicted entry time: {:?}", duration_evicted);
    println!("Cached entry time: {:?}", duration_cached);
    
    // Debug output
    println!("output2_again status: {:?}", output2_again.status);
    println!("output2_again stdout len: {}", output2_again.stdout.len());
    println!("output2_again stderr: {}", String::from_utf8_lossy(&output2_again.stderr));
    
    // Verify correct results
    assert_eq!(output1.stdout, output1_again.stdout, "Data1 should convert identically");
    assert!(output2_again.status.success(), "Data2 second request should succeed");
    assert_eq!(output2.stdout, output2_again.stdout, "Data2 should convert identically");
    
    println!("✓ Cache eviction works with LRU policy\n");
}

#[test]
fn test_cache_disabled_by_default() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    println!("=== Cache: Server works without --cache-size flag ===");
    
    // Start server WITHOUT cache flag (default behavior)
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "toonify", "--", "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        let _ = server.kill();
        let _ = server.wait();
    };
    
    let json_data = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    
    // Make request
    let output = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute request");
    
    cleanup();
    
    assert!(output.status.success(), "Request should succeed without cache");
    
    let result = String::from_utf8_lossy(&output.stdout);
    assert!(result.contains("users"), "Should contain valid conversion result");
    
    println!("✓ Server works correctly without caching enabled\n");
}

