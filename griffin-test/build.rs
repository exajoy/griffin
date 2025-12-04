fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        //.file_descriptor_set_path("helloworld_descriptor.bin")
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &["src/test_support/proto/helloworld/helloworld.proto"],
            &["src/test_support/proto"],
        )?;
    Ok(())
}
