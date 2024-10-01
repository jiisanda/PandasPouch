// build.rs
// This file tells tonic-build to compile the protobuf when building the project

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/pandas_pouch.proto")
    .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    Ok(())
}
