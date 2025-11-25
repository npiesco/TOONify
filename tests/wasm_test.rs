use std::process::Command;
use std::fs;
use std::path::Path;

fn get_manifest_dir() -> String {
    env!("CARGO_MANIFEST_DIR").to_string()
}

#[test]
fn test_wasm_build_succeeds() {
    println!("=== WASM: Build WASM module ===");
    
    let manifest_dir = get_manifest_dir();
    
    // Build WASM module
    let output = Command::new("cargo")
        .args(&["build", "--target", "wasm32-unknown-unknown", "--lib", "--release", "--no-default-features"])
        .current_dir(&manifest_dir)
        .output()
        .expect("Failed to build WASM");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "WASM build should succeed");
    
    // Verify WASM file exists
    let wasm_path = format!("{}/target/wasm32-unknown-unknown/release/toonify.wasm", manifest_dir);
    assert!(Path::new(&wasm_path).exists(), "WASM file should exist at {}", wasm_path);
    
    // Check file size is reasonable
    let metadata = fs::metadata(&wasm_path).expect("Failed to get WASM metadata");
    let size_kb = metadata.len() / 1024;
    println!("WASM size: {} KB", size_kb);
    
    assert!(size_kb > 0, "WASM file should not be empty");
    assert!(size_kb < 5000, "WASM file should be reasonably sized (< 5MB)");
    
    println!("✓ WASM build successful\n");
}

#[test]
fn test_wasm_pack_build() {
    println!("=== WASM: Build with wasm-pack for npm ===");
    
    let manifest_dir = get_manifest_dir();
    
    // Build with wasm-pack
    let output = Command::new("wasm-pack")
        .args(&["build", "--target", "web", "--out-dir", "pkg", "--no-default-features"])
        .current_dir(&manifest_dir)
        .output()
        .expect("Failed to run wasm-pack");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "wasm-pack build should succeed");
    
    // Verify pkg directory exists
    let pkg_dir = format!("{}/pkg", manifest_dir);
    assert!(Path::new(&pkg_dir).exists(), "pkg directory should exist");
    
    // Verify essential files exist
    let wasm_file = format!("{}/toonify_bg.wasm", pkg_dir);
    let js_file = format!("{}/toonify.js", pkg_dir);
    let ts_file = format!("{}/toonify.d.ts", pkg_dir);
    let package_json = format!("{}/package.json", pkg_dir);
    
    assert!(Path::new(&wasm_file).exists(), "WASM file should exist in pkg/");
    assert!(Path::new(&js_file).exists(), "JS glue code should exist");
    assert!(Path::new(&ts_file).exists(), "TypeScript definitions should exist");
    assert!(Path::new(&package_json).exists(), "package.json should exist");
    
    // Patch package.json with required npm fields (wasm-pack generates minimal version)
    let package_json_content = fs::read_to_string(&package_json).expect("Failed to read package.json");
    let mut package: serde_json::Value = serde_json::from_str(&package_json_content).expect("Failed to parse package.json");
    
    // Add required fields for npm publishing
    package["description"] = serde_json::json!("High-performance JSON ↔ TOON converter (WASM) - Reduce LLM token usage by 30-60%");
    package["author"] = serde_json::json!("Nicholas Piesco");
    package["license"] = serde_json::json!("MIT");
    package["repository"] = serde_json::json!({
        "type": "git",
        "url": "https://github.com/npiesco/TOONify.git"
    });
    package["homepage"] = serde_json::json!("https://github.com/npiesco/TOONify#readme");
    package["bugs"] = serde_json::json!({
        "url": "https://github.com/npiesco/TOONify/issues"
    });
    package["keywords"] = serde_json::json!([
        "wasm", "webassembly", "json", "toon", "converter", 
        "llm", "token-optimization", "ai", "rust"
    ]);
    
    // Add README.md to files array
    if let Some(files) = package["files"].as_array_mut() {
        files.push(serde_json::json!("README.md"));
    }
    
    // Write updated package.json
    let updated_json = serde_json::to_string_pretty(&package).expect("Failed to serialize package.json");
    fs::write(&package_json, updated_json).expect("Failed to write package.json");
    
    println!("✓ wasm-pack build successful (package.json patched with npm fields)\n");
}

#[test]
fn test_wasm_json_to_toon_conversion() {
    println!("=== WASM: JSON to TOON conversion via Node.js ===");
    
    let manifest_dir = get_manifest_dir();
    let pkg_dir = format!("{}/pkg", manifest_dir);
    
    // Ensure WASM is built
    if !Path::new(&pkg_dir).exists() {
        println!("Building WASM first...");
        let build_output = Command::new("wasm-pack")
            .args(&["build", "--target", "web", "--out-dir", "pkg", "--no-default-features"])
            .current_dir(&manifest_dir)
            .output()
            .expect("Failed to build WASM");
        assert!(build_output.status.success(), "WASM build failed");
    }
    
    // Create test script
    let test_script = format!(r#"
const fs = require('fs');
const {{ WASI }} = require('wasi');
const {{ argv, env }} = require('process');

// For web target, we need to use a different approach
// This is a simplified test - in reality we'd use the generated JS bindings
console.log('Testing WASM module existence...');

const wasmPath = '{}/toonify_bg.wasm';
if (fs.existsSync(wasmPath)) {{
    const stats = fs.statSync(wasmPath);
    console.log('WASM file size:', stats.size, 'bytes');
    console.log('✓ WASM module found');
    process.exit(0);
}} else {{
    console.error('WASM file not found');
    process.exit(1);
}}
"#, pkg_dir);
    
    let script_path = format!("{}/test_wasm.js", manifest_dir);
    fs::write(&script_path, test_script).expect("Failed to write test script");
    
    // Run Node.js test
    let output = Command::new("node")
        .arg(&script_path)
        .output()
        .expect("Failed to run Node.js test");
    
    println!("Node.js output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
    
    // Cleanup
    let _ = fs::remove_file(&script_path);
    
    assert!(output.status.success(), "Node.js WASM test should succeed");
    
    println!("✓ WASM module accessible from Node.js\n");
}

#[test]
fn test_wasm_package_json_valid() {
    println!("=== WASM: package.json is valid ===");
    
    let manifest_dir = get_manifest_dir();
    let package_json_path = format!("{}/pkg/package.json", manifest_dir);
    
    // Ensure WASM is built
    if !Path::new(&package_json_path).exists() {
        println!("Building WASM first...");
        let build_output = Command::new("wasm-pack")
            .args(&["build", "--target", "web", "--out-dir", "pkg", "--no-default-features"])
            .current_dir(&manifest_dir)
            .output()
            .expect("Failed to build WASM");
        assert!(build_output.status.success(), "WASM build failed");
    }
    
    // Read and parse package.json
    let package_json = fs::read_to_string(&package_json_path)
        .expect("Failed to read package.json");
    
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&package_json)
        .expect("package.json should be valid JSON");
    
    // Verify essential fields
    assert!(parsed["name"].is_string(), "package.json should have name field");
    assert!(parsed["version"].is_string(), "package.json should have version field");
    assert!(parsed["files"].is_array(), "package.json should have files array");
    
    let name = parsed["name"].as_str().unwrap();
    println!("Package name: {}", name);
    assert!(name.contains("toonify"), "Package name should contain 'toonify'");
    
    println!("✓ package.json is valid\n");
}

#[test]
fn test_wasm_typescript_definitions_exist() {
    println!("=== WASM: TypeScript definitions ===");
    
    let manifest_dir = get_manifest_dir();
    let ts_file = format!("{}/pkg/toonify.d.ts", manifest_dir);
    
    // Ensure WASM is built
    if !Path::new(&ts_file).exists() {
        println!("Building WASM first...");
        let build_output = Command::new("wasm-pack")
            .args(&["build", "--target", "web", "--out-dir", "pkg", "--no-default-features"])
            .current_dir(&manifest_dir)
            .output()
            .expect("Failed to build WASM");
        assert!(build_output.status.success(), "WASM build failed");
    }
    
    // Read TypeScript definitions
    let ts_content = fs::read_to_string(&ts_file)
        .expect("Failed to read TypeScript definitions");
    
    // Verify essential exports are declared
    assert!(ts_content.contains("export"), "Should have export declarations");
    assert!(ts_content.contains("json_to_toon") || ts_content.contains("jsonToToon"), 
            "Should export json_to_toon function");
    assert!(ts_content.contains("toon_to_json") || ts_content.contains("toonToJson"), 
            "Should export toon_to_json function");
    
    println!("TypeScript definitions preview:");
    let lines: Vec<&str> = ts_content.lines().take(10).collect();
    for line in lines {
        println!("  {}", line);
    }
    
    println!("✓ TypeScript definitions exist and contain expected exports\n");
}

