use std::io::Write;

const PROTO_CONTENT: &str = r#"syntax = "proto3";

package tmux_gateway;

service TmuxGateway {
  rpc Ls(LsRequest) returns (LsResponse);
}

message LsRequest {}

message LsResponse {
  repeated TmuxSession sessions = 1;
}

message TmuxSession {
  string name = 1;
  uint32 windows = 2;
  string created = 3;
  bool attached = 4;
}"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Write proto file to OUT_DIR (no external .proto source needed)
    let proto_path = out_dir.join("tmux_gateway.proto");
    std::fs::write(&proto_path, PROTO_CONTENT)?;

    // Compile the proto from OUT_DIR
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("tmux_gateway_descriptor.bin"))
        .compile_protos(
            &[proto_path.to_str().unwrap()],
            &[out_dir.to_str().unwrap()],
        )?;

    // Export proto content as a Rust constant for runtime access
    let mut f = std::fs::File::create(out_dir.join("proto_content.rs"))?;
    writeln!(f, "pub const PROTO_CONTENT: &str = {:?};", PROTO_CONTENT)?;

    Ok(())
}
