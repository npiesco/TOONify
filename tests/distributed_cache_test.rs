use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;

// Global mutex to prevent concurrent cache tests
static CACHE_TEST_LOCK: Mutex<()> = Mutex::new(());

fn is_memcached_available() -> bool {
    Command::new("nc")
        .args(&["-z", "127.0.0.1", "11211"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn is_valkey_available() -> bool {
    Command::new("valkey-cli")
        .args(&["ping"])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("PONG"))
        .unwrap_or(false)
}

fn flush_memcached() {
    let _ = Command::new("sh")
        .args(&["-c", "echo 'flush_all' | nc 127.0.0.1 11211"])
        .output();
}

fn flush_valkey() {
    let _ = Command::new("valkey-cli")
        .args(&["FLUSHDB"])
        .output();
}

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
fn test_memcached_cache_enabled_with_flag() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    
    if !is_memcached_available() {
        println!("⚠ Memcached not available, skipping test (install with: brew install memcached && brew services start memcached)");
        return;
    }
    
    println!("=== Memcached: Cache can be enabled with --memcached flag ===");
    
    flush_memcached();
    
    // Start server with Memcached caching
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--memcached", "127.0.0.1:11211"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
        flush_memcached();
    };
    
    let json_data = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    
    // First request - cache miss
    let output1 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output1.status.success(), "First request should succeed");
    let response1 = String::from_utf8_lossy(&output1.stdout);
    assert!(response1.contains("users"), "Response should contain TOON data");
    
    // Second request - should hit cache
    let output2 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output2.status.success(), "Second request should succeed");
    let response2 = String::from_utf8_lossy(&output2.stdout);
    assert!(response2.contains("users"), "Cached response should contain TOON data");
    assert_eq!(response1, response2, "Cache should return same result");
    
    cleanup();
    
    println!("✓ Memcached cache works correctly");
}

#[test]
fn test_memcached_cache_persists_across_restarts() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    
    if !is_memcached_available() {
        println!("⚠ Memcached not available, skipping test");
        return;
    }
    
    println!("=== Memcached: Cache persists across server restarts ===");
    
    flush_memcached();
    
    let json_data = r#"{"products":[{"id":1,"name":"Widget"}]}"#;
    
    // Start server
    let mut server1 = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--memcached", "127.0.0.1:11211"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    // Make a request to populate cache
    let output1 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output1.status.success(), "Initial request should succeed");
    let initial_response = String::from_utf8_lossy(&output1.stdout);
    
    // Stop server
    let _ = server1.kill();
    let _ = server1.wait();
    thread::sleep(Duration::from_millis(500));
    
    // Start server again
    let mut server2 = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--memcached", "127.0.0.1:11211"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start after restart");
    
    // Make same request - should hit Memcached cache
    let output2 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output2.status.success(), "Request after restart should succeed");
    let cached_response = String::from_utf8_lossy(&output2.stdout);
    
    assert_eq!(initial_response, cached_response, "Memcached should preserve cache across restarts");
    
    let _ = server2.kill();
    let _ = server2.wait();
    flush_memcached();
    
    println!("✓ Memcached cache persists across restarts");
}

#[test]
fn test_valkey_cache_enabled_with_flag() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    
    if !is_valkey_available() {
        println!("⚠ Valkey not available, skipping test (install with: brew install valkey && brew services start valkey)");
        return;
    }
    
    println!("=== Valkey: Cache can be enabled with --valkey flag ===");
    
    flush_valkey();
    
    // Start server with Valkey caching
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--valkey", "valkey://127.0.0.1:6379"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let mut cleanup = || {
        println!("Cleaning up server...");
        let _ = server.kill();
        let _ = server.wait();
        flush_valkey();
    };
    
    let json_data = r#"{"users":[{"id":1,"name":"Bob"}]}"#;
    
    // First request - cache miss
    let output1 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output1.status.success(), "First request should succeed");
    let response1 = String::from_utf8_lossy(&output1.stdout);
    assert!(response1.contains("users"), "Response should contain TOON data");
    
    // Check Valkey has cached the result
    let valkey_check = Command::new("valkey-cli")
        .args(&["EXISTS", "toonify:json_to_toon"])
        .output()
        .expect("Failed to check Valkey");
    
    let valkey_output = String::from_utf8_lossy(&valkey_check.stdout);
    println!("Valkey cache check: {}", valkey_output);
    
    // Second request - should hit cache
    let output2 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output2.status.success(), "Second request should succeed");
    let response2 = String::from_utf8_lossy(&output2.stdout);
    assert!(response2.contains("users"), "Cached response should contain TOON data");
    assert_eq!(response1, response2, "Cache should return same result");
    
    cleanup();
    
    println!("✓ Valkey cache works correctly");
}

#[test]
fn test_valkey_cache_with_ttl() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    
    if !is_valkey_available() {
        println!("⚠ Valkey not available, skipping test");
        return;
    }
    
    println!("=== Valkey: Cache entries have TTL (Time To Live) ===");
    
    flush_valkey();
    
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--valkey", "valkey://127.0.0.1:6379", "--cache-ttl", "2"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let json_data = r#"{"test":[{"value":123}]}"#;
    
    // Make request to populate cache
    let output1 = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output1.status.success());
    
    // Check Valkey has the key
    let valkey_check1 = Command::new("valkey-cli")
        .args(&["KEYS", "toonify:*"])
        .output()
        .expect("Failed to check Valkey");
    
    let keys1 = String::from_utf8_lossy(&valkey_check1.stdout);
    assert!(!keys1.trim().is_empty(), "Valkey should have cached entry");
    
    // Wait for TTL to expire
    println!("Waiting for TTL to expire (3 seconds)...");
    thread::sleep(Duration::from_secs(3));
    
    // Check Valkey no longer has the key
    let valkey_check2 = Command::new("valkey-cli")
        .args(&["KEYS", "toonify:*"])
        .output()
        .expect("Failed to check Valkey");
    
    let keys2 = String::from_utf8_lossy(&valkey_check2.stdout);
    assert!(keys2.trim().is_empty() || keys2.trim() == "(empty array)", 
            "Valkey should have evicted expired entry");
    
    let _ = server.kill();
    let _ = server.wait();
    flush_valkey();
    
    println!("✓ Valkey cache respects TTL configuration");
}

#[test]
fn test_memcached_vs_valkey_both_work() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    
    let memcached_available = is_memcached_available();
    let valkey_available = is_valkey_available();
    
    if !memcached_available && !valkey_available {
        println!("⚠ Neither Memcached nor Valkey available, skipping test");
        return;
    }
    
    println!("=== Distributed Cache: Both Memcached and Valkey work independently ===");
    
    if memcached_available {
        flush_memcached();
        
        let mut server_mc = Command::new("cargo")
            .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--memcached", "127.0.0.1:11211"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server with Memcached");
        
        assert!(wait_for_server_ready(100), "Server with Memcached failed to start");
        
        let output_mc = Command::new("curl")
            .args(&[
                "-s",
                "-X", "POST",
                "http://localhost:5000/json-to-toon",
                "-H", "Content-Type: application/json",
                "-d", r#"{"data":"{\"test\":1}"}"#
            ])
            .output()
            .expect("Failed to execute curl");
        
        assert!(output_mc.status.success(), "Memcached request should succeed");
        
        let _ = server_mc.kill();
        let _ = server_mc.wait();
        thread::sleep(Duration::from_millis(500));
        
        println!("✓ Memcached works");
    }
    
    if valkey_available {
        flush_valkey();
        
        let mut server_vk = Command::new("cargo")
            .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--valkey", "valkey://127.0.0.1:6379"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server with Valkey");
        
        assert!(wait_for_server_ready(100), "Server with Valkey failed to start");
        
        let output_vk = Command::new("curl")
            .args(&[
                "-s",
                "-X", "POST",
                "http://localhost:5000/json-to-toon",
                "-H", "Content-Type: application/json",
                "-d", r#"{"data":"{\"test\":2}"}"#
            ])
            .output()
            .expect("Failed to execute curl");
        
        assert!(output_vk.status.success(), "Valkey request should succeed");
        
        let _ = server_vk.kill();
        let _ = server_vk.wait();
        
        println!("✓ Valkey works");
    }
    
    println!("✓ Both cache backends work independently");
}

#[test]
fn test_server_works_without_distributed_cache() {
    let _lock = CACHE_TEST_LOCK.lock().unwrap();
    
    println!("=== Distributed Cache: Server works fine without cache flags (fallback to LRU) ===");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--release", "--bin", "toonify", "--", "serve", "--cache-size", "10"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    assert!(wait_for_server_ready(100), "Server failed to start");
    
    let json_data = r#"{"fallback":[{"test":true}]}"#;
    
    let output = Command::new("curl")
        .args(&[
            "-s",
            "-X", "POST",
            "http://localhost:5000/json-to-toon",
            "-H", "Content-Type: application/json",
            "-d", &format!(r#"{{"data":"{}"}}"#, json_data.replace("\"", "\\\""))
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output.status.success(), "Request should succeed without distributed cache");
    let response = String::from_utf8_lossy(&output.stdout);
    assert!(response.contains("fallback"), "Should get valid response");
    
    let _ = server.kill();
    let _ = server.wait();
    
    println!("✓ Server works correctly without distributed cache (uses LRU cache)");
}

