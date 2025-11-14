use std::process::Command;
use std::fs;

fn get_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/toonify", manifest_dir)
}

#[test]
fn test_compress_toon_data() {
    println!("=== Compression: Compress TOON data ===");
    
    // Create test TOON data
    let toon_data = r#"users[3]{id,name,email}:
1,Alice,alice@example.com
2,Bob,bob@example.com
3,Charlie,charlie@example.com"#;
    
    let input_file = "/tmp/test_compress_input.toon";
    let output_file = "/tmp/test_compress_output.toon.gz";
    
    fs::write(input_file, toon_data).expect("Failed to write input file");
    
    println!("Input size: {} bytes", toon_data.len());
    
    // Compress the TOON data
    let output = Command::new(get_binary_path())
        .args(&["compress", "--input", input_file, "--output", output_file])
        .output()
        .expect("Failed to execute compress command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Compress command should succeed");
    assert!(fs::metadata(output_file).is_ok(), "Compressed file should exist");
    
    let compressed_size = fs::metadata(output_file).unwrap().len();
    println!("Compressed size: {} bytes", compressed_size);
    
    // Compressed size should be smaller than original (for reasonable data)
    assert!(compressed_size < toon_data.len() as u64 || toon_data.len() < 100, 
            "Compressed data should be smaller (or input was too small to compress)");
    
    // Cleanup
    let _ = fs::remove_file(input_file);
    let _ = fs::remove_file(output_file);
    
    println!("✓ TOON compression successful\n");
}

#[test]
fn test_decompress_toon_data() {
    println!("=== Compression: Decompress TOON data ===");
    
    // Create test TOON data
    let original_toon = r#"products[5]{id,name,price,category}:
1,Laptop,999.99,Electronics
2,Mouse,29.99,Electronics
3,Keyboard,79.99,Electronics
4,Monitor,299.99,Electronics
5,Desk,399.99,Furniture"#;
    
    let input_file = "/tmp/test_decompress_input.toon";
    let compressed_file = "/tmp/test_decompress_compressed.toon.gz";
    let output_file = "/tmp/test_decompress_output.toon";
    
    fs::write(input_file, original_toon).expect("Failed to write input file");
    
    // First compress
    let compress_output = Command::new(get_binary_path())
        .args(&["compress", "--input", input_file, "--output", compressed_file])
        .output()
        .expect("Failed to execute compress command");
    
    assert!(compress_output.status.success(), "Compress should succeed");
    println!("Compressed file created");
    
    // Now decompress
    let decompress_output = Command::new(get_binary_path())
        .args(&["decompress", "--input", compressed_file, "--output", output_file])
        .output()
        .expect("Failed to execute decompress command");
    
    println!("Exit status: {}", decompress_output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&decompress_output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&decompress_output.stderr));
    
    assert!(decompress_output.status.success(), "Decompress command should succeed");
    assert!(fs::metadata(output_file).is_ok(), "Decompressed file should exist");
    
    // Read decompressed data and verify it matches original
    let decompressed_data = fs::read_to_string(output_file).expect("Failed to read decompressed file");
    
    println!("Original length: {}", original_toon.len());
    println!("Decompressed length: {}", decompressed_data.len());
    
    assert_eq!(decompressed_data, original_toon, "Decompressed data should match original");
    
    // Cleanup
    let _ = fs::remove_file(input_file);
    let _ = fs::remove_file(compressed_file);
    let _ = fs::remove_file(output_file);
    
    println!("✓ TOON decompression successful\n");
}

#[test]
fn test_compress_large_toon_dataset() {
    println!("=== Compression: Large TOON dataset ===");
    
    // Generate large TOON dataset (1000 users)
    let mut toon_lines = vec!["users[1000]{id,name,email,created_at}:".to_string()];
    for i in 0..1000 {
        toon_lines.push(format!(
            "{},User{},user{}@example.com,2024-01-15T10:30:00Z",
            i, i, i
        ));
    }
    let large_toon = toon_lines.join("\n");
    
    let input_file = "/tmp/test_large_compress.toon";
    let output_file = "/tmp/test_large_compress.toon.gz";
    
    fs::write(input_file, &large_toon).expect("Failed to write large input file");
    
    let original_size = large_toon.len();
    println!("Original size: {} bytes", original_size);
    
    // Compress
    let output = Command::new(get_binary_path())
        .args(&["compress", "--input", input_file, "--output", output_file])
        .output()
        .expect("Failed to execute compress command");
    
    assert!(output.status.success(), "Large compress should succeed");
    
    let compressed_size = fs::metadata(output_file).unwrap().len() as usize;
    println!("Compressed size: {} bytes", compressed_size);
    
    let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
    println!("Compression ratio: {:.2}%", compression_ratio);
    
    // Should achieve at least 30% compression on repetitive data
    assert!(compression_ratio > 30.0, 
            "Should achieve >30% compression on large repetitive data (got {:.2}%)", 
            compression_ratio);
    
    // Cleanup
    let _ = fs::remove_file(input_file);
    let _ = fs::remove_file(output_file);
    
    println!("✓ Large dataset compression successful\n");
}

#[test]
fn test_compress_decompress_roundtrip() {
    println!("=== Compression: Roundtrip test ===");
    
    let original = r#"inventory[10]{sku,product,quantity,location}:
SKU001,Widget A,150,Warehouse-1
SKU002,Widget B,200,Warehouse-1
SKU003,Gadget C,75,Warehouse-2
SKU004,Tool D,300,Warehouse-3
SKU005,Part E,500,Warehouse-1
SKU006,Component F,100,Warehouse-2
SKU007,Device G,50,Warehouse-3
SKU008,Item H,250,Warehouse-1
SKU009,Object I,175,Warehouse-2
SKU010,Thing J,225,Warehouse-3"#;
    
    let input_file = "/tmp/test_roundtrip_input.toon";
    let compressed_file = "/tmp/test_roundtrip.toon.gz";
    let output_file = "/tmp/test_roundtrip_output.toon";
    
    fs::write(input_file, original).expect("Failed to write input");
    
    // Compress
    let compress = Command::new(get_binary_path())
        .args(&["compress", "--input", input_file, "--output", compressed_file])
        .output()
        .expect("Failed to compress");
    
    assert!(compress.status.success(), "Compress failed");
    
    // Decompress
    let decompress = Command::new(get_binary_path())
        .args(&["decompress", "--input", compressed_file, "--output", output_file])
        .output()
        .expect("Failed to decompress");
    
    assert!(decompress.status.success(), "Decompress failed");
    
    // Verify roundtrip
    let final_data = fs::read_to_string(output_file).expect("Failed to read output");
    assert_eq!(final_data, original, "Roundtrip should preserve data exactly");
    
    // Cleanup
    let _ = fs::remove_file(input_file);
    let _ = fs::remove_file(compressed_file);
    let _ = fs::remove_file(output_file);
    
    println!("✓ Compress/decompress roundtrip successful\n");
}

#[test]
fn test_compress_via_stdin_stdout() {
    println!("=== Compression: stdin/stdout ===");
    
    let toon_data = "items[3]{id,name}:\n1,Item A\n2,Item B\n3,Item C";
    
    // Compress via stdin/stdout
    let compress_output = Command::new(get_binary_path())
        .args(&["compress"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(toon_data.as_bytes())?;
            child.wait_with_output()
        })
        .expect("Failed to compress via stdin");
    
    println!("Compress exit status: {}", compress_output.status);
    println!("Compress stderr: {}", String::from_utf8_lossy(&compress_output.stderr));
    
    assert!(compress_output.status.success(), "Compress via stdin should succeed");
    assert!(!compress_output.stdout.is_empty(), "Should have compressed output");
    
    let compressed_bytes = compress_output.stdout;
    println!("Compressed {} bytes to {} bytes", toon_data.len(), compressed_bytes.len());
    
    // Decompress via stdin/stdout
    let decompress_output = Command::new(get_binary_path())
        .args(&["decompress"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(&compressed_bytes)?;
            child.wait_with_output()
        })
        .expect("Failed to decompress via stdin");
    
    println!("Decompress exit status: {}", decompress_output.status);
    println!("Decompress stderr: {}", String::from_utf8_lossy(&decompress_output.stderr));
    
    assert!(decompress_output.status.success(), "Decompress via stdin should succeed");
    
    let decompressed = String::from_utf8_lossy(&decompress_output.stdout);
    assert_eq!(decompressed, toon_data, "stdin/stdout roundtrip should preserve data");
    
    println!("✓ stdin/stdout compression successful\n");
}

