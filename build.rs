fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build gRPC protobuf files
    tonic_build::compile_protos("proto/converter.proto")?;
    
    // UniFFI 0.29+ uses proc-macros, no build script scaffolding needed
    // Scaffolding is generated via #[uniffi::export] annotations at compile time
    
    Ok(())
}
