use protox::file::{ChainFileResolver, File, FileResolver, GoogleFileResolver};

struct StringResolver {
    name: String,
    source: String,
}

impl FileResolver for StringResolver {
    fn open_file(&self, name: &str) -> Result<File, protox::Error> {
        if name == self.name {
            File::from_source(name, &self.source)
        } else {
            Err(protox::Error::file_not_found(name))
        }
    }
}

pub fn compile_proto(proto_content: &str) -> prost_types::FileDescriptorSet {
    let mut resolver = ChainFileResolver::new();
    resolver.add(StringResolver {
        name: "tmux_gateway.proto".to_string(),
        source: proto_content.to_string(),
    });
    resolver.add(GoogleFileResolver::new());

    protox::Compiler::with_file_resolver(resolver)
        .open_file("tmux_gateway.proto")
        .expect("failed to compile proto")
        .file_descriptor_set()
}
