use std::fmt::Write as FmtWrite;
use std::io::Write;

#[derive(Clone, Copy)]
enum FieldType {
    String,
    Uint32,
    Bool,
}

impl FieldType {
    fn proto_name(self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Uint32 => "uint32",
            FieldType::Bool => "bool",
        }
    }
}

struct Field {
    name: &'static str,
    ty: FieldType,
    repeated: bool,
    message_type: Option<&'static str>,
}

impl Field {
    fn simple(name: &'static str, ty: FieldType) -> Self {
        Self {
            name,
            ty,
            repeated: false,
            message_type: None,
        }
    }

    fn repeated_message(name: &'static str, message_type: &'static str) -> Self {
        Self {
            name,
            ty: FieldType::String, // unused when message_type is set
            repeated: true,
            message_type: Some(message_type),
        }
    }

    fn type_str(&self) -> &str {
        self.message_type.unwrap_or_else(|| self.ty.proto_name())
    }
}

struct Message {
    name: &'static str,
    fields: Vec<Field>,
}

struct Rpc {
    name: &'static str,
    request: &'static str,
    response: &'static str,
}

struct Service {
    name: &'static str,
    rpcs: Vec<Rpc>,
}

fn generate_proto(package: &str, messages: &[Message], services: &[Service]) -> String {
    let mut proto = String::new();
    writeln!(proto, "syntax = \"proto3\";").unwrap();
    writeln!(proto).unwrap();
    writeln!(proto, "package {package};").unwrap();

    for service in services {
        writeln!(proto).unwrap();
        writeln!(proto, "service {} {{", service.name).unwrap();
        for rpc in &service.rpcs {
            writeln!(
                proto,
                "  rpc {}({}) returns ({});",
                rpc.name, rpc.request, rpc.response
            )
            .unwrap();
        }
        writeln!(proto, "}}").unwrap();
    }

    for message in messages {
        writeln!(proto).unwrap();
        writeln!(proto, "message {} {{", message.name).unwrap();
        for (i, field) in message.fields.iter().enumerate() {
            let repeated = if field.repeated { "repeated " } else { "" };
            writeln!(
                proto,
                "  {}{} {} = {};",
                repeated,
                field.type_str(),
                field.name,
                i + 1
            )
            .unwrap();
        }
        writeln!(proto, "}}").unwrap();
    }

    proto
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let messages = vec![
        Message {
            name: "LsRequest",
            fields: vec![],
        },
        Message {
            name: "LsResponse",
            fields: vec![Field::repeated_message("sessions", "TmuxSession")],
        },
        Message {
            name: "TmuxSession",
            fields: vec![
                Field::simple("name", FieldType::String),
                Field::simple("windows", FieldType::Uint32),
                Field::simple("created", FieldType::String),
                Field::simple("attached", FieldType::Bool),
            ],
        },
        Message {
            name: "NewSessionRequest",
            fields: vec![Field::simple("name", FieldType::String)],
        },
        Message {
            name: "NewSessionResponse",
            fields: vec![Field::simple("name", FieldType::String)],
        },
    ];

    let services = vec![Service {
        name: "TmuxGateway",
        rpcs: vec![
            Rpc {
                name: "Ls",
                request: "LsRequest",
                response: "LsResponse",
            },
            Rpc {
                name: "NewSession",
                request: "NewSessionRequest",
                response: "NewSessionResponse",
            },
        ],
    }];

    let proto_content = generate_proto("tmux_gateway", &messages, &services);

    // Write proto file to OUT_DIR
    let proto_path = out_dir.join("tmux_gateway.proto");
    std::fs::write(&proto_path, &proto_content)?;

    // Compile the proto from OUT_DIR
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("tmux_gateway_descriptor.bin"))
        .compile_protos(
            &[proto_path.to_str().unwrap()],
            &[out_dir.to_str().unwrap()],
        )?;

    // Export proto content as a Rust constant for runtime access
    let mut f = std::fs::File::create(out_dir.join("proto_content.rs"))?;
    writeln!(f, "pub const PROTO_CONTENT: &str = {:?};", proto_content)?;

    Ok(())
}
