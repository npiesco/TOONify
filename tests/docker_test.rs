use std::process::Command;
use std::fs;
use std::path::PathBuf;

fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_dockerfile_exists() {
    println!("=== Docker: Dockerfile Exists ===");
    
    let mut dockerfile = get_project_root();
    dockerfile.push("Dockerfile");
    
    println!("Checking for Dockerfile at: {:?}", dockerfile);
    assert!(dockerfile.exists(), "Dockerfile should exist in project root");
    
    let content = fs::read_to_string(&dockerfile).expect("Failed to read Dockerfile");
    println!("Dockerfile content ({} bytes):\n{}\n", content.len(), content);
    
    assert!(content.contains("FROM"), "Dockerfile should have FROM instruction");
    assert!(content.contains("rust") || content.contains("alpine"), 
            "Dockerfile should use rust or alpine base image");
    
    println!("✓ Dockerfile exists and looks valid\n");
}

#[test]
fn test_docker_build() {
    if !docker_available() {
        println!("Skipping test: Docker not available");
        return;
    }
    
    println!("=== Docker: Build Image ===");
    
    let project_root = get_project_root();
    println!("Building from: {:?}", project_root);
    
    let output = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg("toonify:test")
        .arg("-f")
        .arg("Dockerfile")
        .arg(".")
        .current_dir(&project_root)
        .output()
        .expect("Failed to execute docker build");
    
    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Docker build should succeed");
    println!("✓ Docker image built successfully\n");
}

#[test]
fn test_docker_run_convert_json() {
    if !docker_available() {
        println!("Skipping test: Docker not available");
        return;
    }
    
    println!("=== Docker: Run Convert JSON ===");
    
    let json_input = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
    
    println!("Input JSON: {}", json_input);
    
    let mut child = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("toonify:test")
        .arg("convert")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn docker run");
    
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json_input.as_bytes()).expect("Failed to write to stdin");
    }
    
    let output = child.wait_with_output().expect("Failed to wait for docker");
    
    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success(), "Docker container should run successfully");
    
    let toon_output = String::from_utf8_lossy(&output.stdout);
    assert!(!toon_output.is_empty(), "Should produce TOON output");
    assert!(toon_output.contains("users"), "TOON should contain 'users'");
    assert!(toon_output.contains("Alice"), "TOON should contain data");
    
    println!("✓ Docker container converted JSON successfully\n");
}

#[test]
fn test_docker_run_server() {
    if !docker_available() {
        println!("Skipping test: Docker not available");
        return;
    }
    
    println!("=== Docker: Run Server ===");
    
    // Start container in background
    let start_output = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg("toonify_test_server")
        .arg("-p")
        .arg("15000:5000")
        .arg("toonify:test")
        .arg("serve")
        .output()
        .expect("Failed to start docker container");
    
    println!("Container start status: {}", start_output.status);
    let container_id = String::from_utf8_lossy(&start_output.stdout).trim().to_string();
    println!("Container ID: {}", container_id);
    
    assert!(start_output.status.success(), "Container should start");
    
    // Wait for server to be ready
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    // Test health endpoint
    let health_check = Command::new("curl")
        .arg("-f")
        .arg("http://localhost:15000/")
        .output();
    
    let cleanup = || {
        println!("Cleaning up container...");
        let _ = Command::new("docker")
            .arg("stop")
            .arg("toonify_test_server")
            .output();
        let _ = Command::new("docker")
            .arg("rm")
            .arg("toonify_test_server")
            .output();
    };
    
    if let Ok(output) = health_check {
        println!("Health check status: {}", output.status);
        println!("Health check response: {}", String::from_utf8_lossy(&output.stdout));
        
        cleanup();
        
        assert!(output.status.success(), "Health check should succeed");
        println!("✓ Docker server running successfully\n");
    } else {
        cleanup();
        panic!("Failed to check server health");
    }
}

#[test]
fn test_dockerignore_exists() {
    println!("=== Docker: .dockerignore Exists ===");
    
    let mut dockerignore = get_project_root();
    dockerignore.push(".dockerignore");
    
    println!("Checking for .dockerignore at: {:?}", dockerignore);
    assert!(dockerignore.exists(), ".dockerignore should exist in project root");
    
    let content = fs::read_to_string(&dockerignore).expect("Failed to read .dockerignore");
    println!(".dockerignore content:\n{}\n", content);
    
    assert!(content.contains("target"), ".dockerignore should exclude target directory");
    
    println!("✓ .dockerignore exists and looks valid\n");
}

#[test]
fn test_docker_image_size() {
    if !docker_available() {
        println!("Skipping test: Docker not available");
        return;
    }
    
    println!("=== Docker: Image Size Check ===");
    
    let output = Command::new("docker")
        .arg("images")
        .arg("toonify:test")
        .arg("--format")
        .arg("{{.Size}}")
        .output()
        .expect("Failed to check image size");
    
    let size_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("Image size: {}", size_str);
    
    assert!(!size_str.is_empty(), "Should report image size");
    
    // Image should be reasonably sized (less than 500MB for a Rust app)
    let size_mb = if size_str.contains("MB") {
        size_str.replace("MB", "").trim().parse::<f64>().unwrap_or(0.0)
    } else if size_str.contains("GB") {
        size_str.replace("GB", "").trim().parse::<f64>().unwrap_or(0.0) * 1024.0
    } else {
        0.0
    };
    
    println!("Image size: {:.2} MB", size_mb);
    assert!(size_mb > 0.0, "Should have valid size");
    assert!(size_mb < 500.0, "Image should be less than 500MB (got {:.2}MB)", size_mb);
    
    println!("✓ Docker image size is reasonable\n");
}

