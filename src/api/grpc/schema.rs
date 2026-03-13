use prost_types::FileDescriptorSet;

pub fn compile_proto(proto_content: &str) -> FileDescriptorSet {
    protox::Compiler::new(["."])
        .unwrap()
        .open_file_from_string("tmux_gateway.proto", proto_content)
        .unwrap()
        .file_descriptor_set()
}
