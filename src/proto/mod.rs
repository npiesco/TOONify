// Pre-generated protobuf code for gRPC service
// To regenerate: PROTO_REGEN=1 cargo build
// Then copy target/.../out/converter.rs to src/proto/generated.rs

// Only include generated code when not building for WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod generated {
    include!("generated.rs");
}
