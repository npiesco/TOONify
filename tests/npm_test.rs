use std::process::Command;
use std::path::PathBuf;
use serde_json::Value;
use std::fs;

fn get_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_npm_package_json_complete() {
    println!("=== NPM: package.json has required fields for publishing ===");
    
    let manifest_dir = get_manifest_dir();
    let pkg_dir = manifest_dir.join("pkg");
    let package_json_path = pkg_dir.join("package.json");
    
    assert!(package_json_path.exists(), "pkg/package.json should exist (run wasm-pack build first)");
    
    let content = fs::read_to_string(&package_json_path)
        .expect("Failed to read package.json");
    
    let package: Value = serde_json::from_str(&content)
        .expect("package.json should be valid JSON");
    
    // Required fields for npm publishing
    assert!(package.get("name").is_some(), "package.json should have 'name' field");
    assert_eq!(package["name"], "toonify", "'name' should be 'toonify'");
    
    assert!(package.get("version").is_some(), "package.json should have 'version' field");
    
    assert!(package.get("description").is_some(), "package.json should have 'description' field");
    assert!(!package["description"].as_str().unwrap().is_empty(), "'description' should not be empty");
    
    assert!(package.get("author").is_some(), "package.json should have 'author' field");
    
    assert!(package.get("license").is_some(), "package.json should have 'license' field");
    assert_eq!(package["license"], "MIT", "'license' should be 'MIT'");
    
    assert!(package.get("repository").is_some(), "package.json should have 'repository' field");
    let repo = package["repository"].as_object().expect("'repository' should be an object");
    assert!(repo.get("type").is_some(), "'repository' should have 'type' field");
    assert!(repo.get("url").is_some(), "'repository' should have 'url' field");
    assert!(repo["url"].as_str().unwrap().contains("github.com"), "'repository.url' should be a GitHub URL");
    
    assert!(package.get("keywords").is_some(), "package.json should have 'keywords' field");
    let keywords = package["keywords"].as_array().expect("'keywords' should be an array");
    assert!(!keywords.is_empty(), "'keywords' should not be empty");
    assert!(keywords.iter().any(|k| k.as_str() == Some("wasm")), "'keywords' should include 'wasm'");
    
    assert!(package.get("homepage").is_some(), "package.json should have 'homepage' field");
    
    assert!(package.get("bugs").is_some(), "package.json should have 'bugs' field");
    
    println!("✓ package.json has all required fields for npm publishing");
}

#[test]
fn test_npm_package_files_exist() {
    println!("=== NPM: All package files exist ===");
    
    let manifest_dir = get_manifest_dir();
    let pkg_dir = manifest_dir.join("pkg");
    
    assert!(pkg_dir.exists(), "pkg/ directory should exist");
    
    let wasm_file = pkg_dir.join("toonify_bg.wasm");
    assert!(wasm_file.exists(), "toonify_bg.wasm should exist");
    
    let js_file = pkg_dir.join("toonify.js");
    assert!(js_file.exists(), "toonify.js should exist");
    
    let dts_file = pkg_dir.join("toonify.d.ts");
    assert!(dts_file.exists(), "toonify.d.ts should exist");
    
    let package_json = pkg_dir.join("package.json");
    assert!(package_json.exists(), "package.json should exist");
    
    // Check file sizes
    let wasm_metadata = fs::metadata(&wasm_file).expect("Failed to get WASM metadata");
    assert!(wasm_metadata.len() > 50_000, "WASM file should be at least 50KB");
    assert!(wasm_metadata.len() < 500_000, "WASM file should be less than 500KB");
    
    println!("✓ All package files exist and have reasonable sizes");
}

#[test]
fn test_npm_package_typescript_definitions() {
    println!("=== NPM: TypeScript definitions are valid ===");
    
    let manifest_dir = get_manifest_dir();
    let dts_file = manifest_dir.join("pkg").join("toonify.d.ts");
    
    assert!(dts_file.exists(), "toonify.d.ts should exist");
    
    let content = fs::read_to_string(&dts_file)
        .expect("Failed to read TypeScript definitions");
    
    // Check for expected exports
    assert!(content.contains("export function json_to_toon"), "Should export json_to_toon function");
    assert!(content.contains("export function toon_to_json"), "Should export toon_to_json function");
    assert!(content.contains("export function init"), "Should export init function");
    
    // Check for parameter types
    assert!(content.contains("string"), "Should have string types");
    
    println!("✓ TypeScript definitions contain expected exports");
}

#[test]
fn test_npm_install_locally() {
    println!("=== NPM: Package can be installed locally ===");
    
    let manifest_dir = get_manifest_dir();
    let pkg_dir = manifest_dir.join("pkg");
    
    assert!(pkg_dir.exists(), "pkg/ directory should exist");
    
    // Create a temporary test directory
    let test_dir = manifest_dir.join("test_npm_install");
    let _ = fs::remove_dir_all(&test_dir); // Clean up if exists
    fs::create_dir(&test_dir).expect("Failed to create test directory");
    
    // Create a minimal package.json
    let test_package_json = r#"{
  "name": "test-toonify-install",
  "version": "1.0.0",
  "type": "module"
}"#;
    fs::write(test_dir.join("package.json"), test_package_json)
        .expect("Failed to write test package.json");
    
    // Try to install the local package
    let output = Command::new("npm")
        .args(&["install", &pkg_dir.to_string_lossy()])
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run npm install");
    
    println!("npm install exit status: {}", output.status);
    if !output.stdout.is_empty() {
        println!("npm install stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("npm install stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    
    assert!(output.status.success(), "npm install should succeed");
    
    // Verify node_modules was created
    let node_modules = test_dir.join("node_modules");
    assert!(node_modules.exists(), "node_modules should be created");
    
    let toonify_module = node_modules.join("toonify");
    assert!(toonify_module.exists(), "toonify module should be installed");
    
    // Check installed files
    assert!(toonify_module.join("toonify_bg.wasm").exists(), "WASM file should be installed");
    assert!(toonify_module.join("toonify.js").exists(), "JS file should be installed");
    assert!(toonify_module.join("toonify.d.ts").exists(), "TypeScript definitions should be installed");
    
    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
    
    println!("✓ Package can be installed locally with npm");
}

#[test]
fn test_npm_readme_exists() {
    println!("=== NPM: README exists for npm package ===");
    
    let manifest_dir = get_manifest_dir();
    let pkg_readme = manifest_dir.join("pkg").join("README.md");
    
    assert!(pkg_readme.exists(), "pkg/README.md should exist for npm publishing");
    
    let content = fs::read_to_string(&pkg_readme)
        .expect("Failed to read pkg/README.md");
    
    assert!(content.contains("TOONify"), "README should mention TOONify");
    assert!(content.contains("WASM") || content.contains("WebAssembly"), "README should mention WASM");
    assert!(content.contains("npm install"), "README should have npm install instructions");
    assert!(content.contains("import"), "README should have import examples");
    
    println!("✓ README exists and contains essential information");
}

