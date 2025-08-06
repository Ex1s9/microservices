fn main() {
    tonic_build::configure()
        .file_descriptor_set_path("../../target/descriptor.bin")
        .compile_protos(&["../../proto/game.proto"], &["../../proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
