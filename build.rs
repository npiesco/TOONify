fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip build steps for WASM target (no gRPC/protobuf support)
    let target = std::env::var("TARGET").unwrap_or_default();

    if !target.contains("wasm32") {
        // Only regenerate protobuf code if proto-regen feature enabled OR PROTO_REGEN=1
        // By default, use pre-generated code in src/proto/generated.rs
        // This way, end users don't need protoc, cmake, or protobuf-src
        #[cfg(feature = "proto-regen")]
        let should_regen = true;
        #[cfg(not(feature = "proto-regen"))]
        let should_regen = std::env::var("PROTO_REGEN").is_ok();

        if should_regen {
            #[cfg(feature = "proto-regen")]
            {
                println!("cargo:warning=Regenerating protobuf code (proto-regen feature enabled)");

                // Set vendored protoc from protobuf-src
                // SAFETY: We're in a build script, single-threaded before main(), safe to set env var
                unsafe {
                    std::env::set_var("PROTOC", protobuf_src::protoc());
                }

                // Build gRPC protobuf files
                tonic_prost_build::compile_protos("proto/converter.proto")?;
            }
            #[cfg(not(feature = "proto-regen"))]
            {
                println!("cargo:warning=PROTO_REGEN=1 set but proto-regen feature not enabled");
                println!("cargo:warning=Run: cargo build --features proto-regen");
            }
        } else {
            println!("cargo:warning=Using pre-generated protobuf code from src/proto/generated.rs");
            println!("cargo:warning=To regenerate: cargo build --features proto-regen");
        }

    // UniFFI 0.29+ uses proc-macros, no build script scaffolding needed
    // Scaffolding is generated via #[uniffi::export] annotations at compile time
    } else {
        println!("cargo:warning=Skipping gRPC/protobuf build for WASM target");
    }

    Ok(())
}
