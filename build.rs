// build.rs
// This file tells tonic-build to compile the protobuf when building the project

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/cache.proto")?;
    Ok(())
}
