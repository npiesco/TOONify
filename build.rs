fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip build steps for WASM target (no gRPC/protobuf support)
    let target = std::env::var("TARGET").unwrap_or_default();
    
    if !target.contains("wasm32") {
        // Build gRPC protobuf files (not for WASM)
        // tonic-build 0.14+ uses tonic_prost_build as a separate crate
        tonic_prost_build::compile_protos("proto/converter.proto")?;
        
        // UniFFI 0.29+ uses proc-macros, no build script scaffolding needed
        // Scaffolding is generated via #[uniffi::export] annotations at compile time
    } else {
        println!("cargo:warning=Skipping gRPC/protobuf build for WASM target");
    }
    
    Ok(())
}
