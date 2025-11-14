// Integration tests for VS Code extension
// Tests verify the extension structure, packaging, and functionality

use std::process::Command;
use std::path::Path;
use std::fs;

fn get_extension_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/vscode-extension", manifest_dir)
}

#[test]
fn test_extension_package_json_exists() {
    println!("=== VS Code Extension: package.json exists ===");
    
    let ext_dir = get_extension_dir();
    let package_json = format!("{}/package.json", ext_dir);
    
    assert!(
        Path::new(&package_json).exists(),
        "Extension package.json should exist at {}",
        package_json
    );
}

#[test]
fn test_extension_package_json_valid() {
    println!("=== VS Code Extension: package.json is valid JSON with required fields ===");
    
    let ext_dir = get_extension_dir();
    let package_json = format!("{}/package.json", ext_dir);
    
    let contents = fs::read_to_string(&package_json)
        .expect("Should be able to read package.json");
    
    let json: serde_json::Value = serde_json::from_str(&contents)
        .expect("package.json should be valid JSON");
    
    // Check required fields for VS Code extension
    assert!(json.get("name").is_some(), "Should have 'name' field");
    assert!(json.get("displayName").is_some(), "Should have 'displayName' field");
    assert!(json.get("description").is_some(), "Should have 'description' field");
    assert!(json.get("version").is_some(), "Should have 'version' field");
    assert!(json.get("publisher").is_some(), "Should have 'publisher' field");
    assert!(json.get("engines").is_some(), "Should have 'engines' field");
    assert!(json.get("categories").is_some(), "Should have 'categories' field");
    assert!(json.get("activationEvents").is_some(), "Should have 'activationEvents' field");
    assert!(json.get("main").is_some(), "Should have 'main' field");
    assert!(json.get("contributes").is_some(), "Should have 'contributes' field");
    
    // Verify specific values
    let name = json["name"].as_str().expect("name should be string");
    assert_eq!(name, "toonify", "Extension name should be 'toonify'");
    
    let engines = &json["engines"];
    assert!(engines.get("vscode").is_some(), "Should specify VS Code engine version");
    
    let contributes = &json["contributes"];
    assert!(contributes.get("commands").is_some(), "Should contribute commands");
    
    println!("✓ package.json is valid with all required fields");
}

#[test]
fn test_extension_entry_point_exists() {
    println!("=== VS Code Extension: Entry point file exists ===");
    
    let ext_dir = get_extension_dir();
    let package_json = format!("{}/package.json", ext_dir);
    
    let contents = fs::read_to_string(&package_json)
        .expect("Should be able to read package.json");
    
    let json: serde_json::Value = serde_json::from_str(&contents)
        .expect("package.json should be valid JSON");
    
    let main = json["main"].as_str().expect("main should be string");
    let entry_point = format!("{}/{}", ext_dir, main);
    
    assert!(
        Path::new(&entry_point).exists(),
        "Extension entry point should exist at {}",
        entry_point
    );
    
    println!("✓ Entry point exists: {}", main);
}

#[test]
fn test_extension_has_required_files() {
    println!("=== VS Code Extension: Required files exist ===");
    
    let ext_dir = get_extension_dir();
    
    let required_files = vec![
        "package.json",
        "README.md",
        ".vscodeignore",
        "tsconfig.json",
    ];
    
    for file in required_files {
        let file_path = format!("{}/{}", ext_dir, file);
        assert!(
            Path::new(&file_path).exists(),
            "Required file should exist: {}",
            file
        );
        println!("✓ Found: {}", file);
    }
}

#[test]
fn test_extension_typescript_compiles() {
    println!("=== VS Code Extension: TypeScript code compiles ===");
    
    let ext_dir = get_extension_dir();
    
    // Install dependencies first
    let npm_install = Command::new("npm")
        .args(&["install"])
        .current_dir(&ext_dir)
        .output()
        .expect("Failed to run npm install");
    
    assert!(
        npm_install.status.success(),
        "npm install should succeed. stderr: {}",
        String::from_utf8_lossy(&npm_install.stderr)
    );
    
    // Compile TypeScript
    let tsc = Command::new("npm")
        .args(&["run", "compile"])
        .current_dir(&ext_dir)
        .output()
        .expect("Failed to run tsc");
    
    assert!(
        tsc.status.success(),
        "TypeScript compilation should succeed. stderr: {}",
        String::from_utf8_lossy(&tsc.stderr)
    );
    
    // Check that out/ directory exists with compiled JS
    let out_dir = format!("{}/out", ext_dir);
    assert!(
        Path::new(&out_dir).exists(),
        "Compiled output directory should exist"
    );
    
    println!("✓ TypeScript compiled successfully");
}

#[test]
fn test_extension_can_be_packaged() {
    println!("=== VS Code Extension: Can be packaged with vsce ===");
    
    let ext_dir = get_extension_dir();
    
    // Install vsce globally if not present
    let vsce_check = Command::new("npx")
        .args(&["vsce", "--version"])
        .output();
    
    if vsce_check.is_err() || !vsce_check.unwrap().status.success() {
        println!("Installing vsce...");
        let install = Command::new("npm")
            .args(&["install", "-g", "@vscode/vsce"])
            .output()
            .expect("Failed to install vsce");
        
        assert!(
            install.status.success(),
            "vsce installation should succeed"
        );
    }
    
    // Package extension
    let package = Command::new("npx")
        .args(&["vsce", "package", "--no-git-tag-version", "--no-update-package-json"])
        .current_dir(&ext_dir)
        .output()
        .expect("Failed to run vsce package");
    
    if !package.status.success() {
        eprintln!("vsce stderr: {}", String::from_utf8_lossy(&package.stderr));
        eprintln!("vsce stdout: {}", String::from_utf8_lossy(&package.stdout));
    }
    
    assert!(
        package.status.success(),
        "Extension packaging should succeed"
    );
    
    // Check that .vsix file was created
    let vsix_files = fs::read_dir(&ext_dir)
        .expect("Should be able to read extension directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "vsix")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    
    assert!(
        !vsix_files.is_empty(),
        "Should have created at least one .vsix file"
    );
    
    println!("✓ Extension packaged successfully");
}

#[test]
fn test_extension_has_conversion_commands() {
    println!("=== VS Code Extension: Has JSON ↔ TOON conversion commands ===");
    
    let ext_dir = get_extension_dir();
    let package_json = format!("{}/package.json", ext_dir);
    
    let contents = fs::read_to_string(&package_json)
        .expect("Should be able to read package.json");
    
    let json: serde_json::Value = serde_json::from_str(&contents)
        .expect("package.json should be valid JSON");
    
    let commands = json["contributes"]["commands"]
        .as_array()
        .expect("commands should be an array");
    
    // Check for required conversion commands
    let command_ids: Vec<&str> = commands
        .iter()
        .filter_map(|cmd| cmd["command"].as_str())
        .collect();
    
    assert!(
        command_ids.contains(&"toonify.jsonToToon"),
        "Should have 'toonify.jsonToToon' command"
    );
    assert!(
        command_ids.contains(&"toonify.toonToJson"),
        "Should have 'toonify.toonToJson' command"
    );
    
    println!("✓ Found conversion commands: {:?}", command_ids);
}

#[test]
fn test_extension_readme_has_usage() {
    println!("=== VS Code Extension: README has usage instructions ===");
    
    let ext_dir = get_extension_dir();
    let readme = format!("{}/README.md", ext_dir);
    
    let contents = fs::read_to_string(&readme)
        .expect("Should be able to read README.md");
    
    // Check for key sections
    assert!(contents.contains("# TOONify"), "README should have title");
    assert!(contents.contains("Features"), "README should describe features");
    assert!(contents.contains("Usage") || contents.contains("How to use"), 
        "README should have usage instructions");
    assert!(contents.contains("JSON") && contents.contains("TOON"), 
        "README should mention JSON and TOON");
    
    println!("✓ README has required sections");
}

