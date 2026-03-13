use std::fmt::Write;

use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    MethodDescriptorProto, ServiceDescriptorProto,
};

pub struct ProtoSchema {
    package: String,
    file_name: String,
    messages: Vec<ProtoMessage>,
    services: Vec<ProtoService>,
}

struct ProtoMessage {
    name: String,
    fields: Vec<ProtoField>,
}

struct ProtoField {
    name: String,
    number: i32,
    field_type: ProtoFieldType,
    repeated: bool,
}

enum ProtoFieldType {
    String,
    Uint32,
    Bool,
    Message(String),
}

struct ProtoService {
    name: String,
    methods: Vec<ProtoMethod>,
}

struct ProtoMethod {
    name: String,
    input_type: String,
    output_type: String,
}

impl ProtoSchema {
    pub fn new(package: &str) -> Self {
        Self {
            file_name: format!("{package}.proto"),
            package: package.to_string(),
            messages: Vec::new(),
            services: Vec::new(),
        }
    }

    pub fn message(mut self, name: &str, f: impl FnOnce(&mut MessageBuilder)) -> Self {
        let mut builder = MessageBuilder { fields: Vec::new() };
        f(&mut builder);
        self.messages.push(ProtoMessage {
            name: name.to_string(),
            fields: builder.fields,
        });
        self
    }

    pub fn service(mut self, name: &str, f: impl FnOnce(&mut ServiceBuilder)) -> Self {
        let mut builder = ServiceBuilder {
            methods: Vec::new(),
        };
        f(&mut builder);
        self.services.push(ProtoService {
            name: name.to_string(),
            methods: builder.methods,
        });
        self
    }

    pub fn proto_string(&self) -> String {
        let mut s = std::string::String::new();
        writeln!(s, "syntax = \"proto3\";").unwrap();
        writeln!(s).unwrap();
        writeln!(s, "package {};", self.package).unwrap();

        for service in &self.services {
            writeln!(s).unwrap();
            writeln!(s, "service {} {{", service.name).unwrap();
            for method in &service.methods {
                writeln!(
                    s,
                    "  rpc {}({}) returns ({});",
                    method.name, method.input_type, method.output_type
                )
                .unwrap();
            }
            writeln!(s, "}}").unwrap();
        }

        for message in &self.messages {
            writeln!(s).unwrap();
            writeln!(s, "message {} {{", message.name).unwrap();
            for field in &message.fields {
                let repeated = if field.repeated { "repeated " } else { "" };
                let type_str = match &field.field_type {
                    ProtoFieldType::String => "string",
                    ProtoFieldType::Uint32 => "uint32",
                    ProtoFieldType::Bool => "bool",
                    ProtoFieldType::Message(name) => name.as_str(),
                };
                writeln!(s, "  {repeated}{type_str} {} = {};", field.name, field.number).unwrap();
            }
            writeln!(s, "}}").unwrap();
        }

        s
    }

    pub fn file_descriptor_set(&self) -> FileDescriptorSet {
        let file = FileDescriptorProto {
            name: Some(self.file_name.clone()),
            package: Some(self.package.clone()),
            syntax: Some("proto3".to_string()),
            message_type: self.messages.iter().map(|m| self.build_descriptor(m)).collect(),
            service: self
                .services
                .iter()
                .map(|s| ServiceDescriptorProto {
                    name: Some(s.name.clone()),
                    method: s
                        .methods
                        .iter()
                        .map(|m| MethodDescriptorProto {
                            name: Some(m.name.clone()),
                            input_type: Some(format!(".{}.{}", self.package, m.input_type)),
                            output_type: Some(format!(".{}.{}", self.package, m.output_type)),
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };

        FileDescriptorSet { file: vec![file] }
    }

    fn build_descriptor(&self, message: &ProtoMessage) -> DescriptorProto {
        DescriptorProto {
            name: Some(message.name.clone()),
            field: message
                .fields
                .iter()
                .map(|f| {
                    let (r#type, type_name) = match &f.field_type {
                        ProtoFieldType::String => (Some(Type::String.into()), None),
                        ProtoFieldType::Uint32 => (Some(Type::Uint32.into()), None),
                        ProtoFieldType::Bool => (Some(Type::Bool.into()), None),
                        ProtoFieldType::Message(name) => (
                            Some(Type::Message.into()),
                            Some(format!(".{}.{}", self.package, name)),
                        ),
                    };
                    FieldDescriptorProto {
                        name: Some(f.name.clone()),
                        number: Some(f.number),
                        r#type,
                        type_name,
                        label: Some(if f.repeated {
                            Label::Repeated.into()
                        } else {
                            Label::Optional.into()
                        }),
                        ..Default::default()
                    }
                })
                .collect(),
            ..Default::default()
        }
    }
}

pub struct MessageBuilder {
    fields: Vec<ProtoField>,
}

impl MessageBuilder {
    pub fn string(&mut self, name: &str, number: i32) {
        self.fields.push(ProtoField {
            name: name.to_string(),
            number,
            field_type: ProtoFieldType::String,
            repeated: false,
        });
    }

    pub fn uint32(&mut self, name: &str, number: i32) {
        self.fields.push(ProtoField {
            name: name.to_string(),
            number,
            field_type: ProtoFieldType::Uint32,
            repeated: false,
        });
    }

    pub fn bool(&mut self, name: &str, number: i32) {
        self.fields.push(ProtoField {
            name: name.to_string(),
            number,
            field_type: ProtoFieldType::Bool,
            repeated: false,
        });
    }

    pub fn repeated_message(&mut self, name: &str, message_type: &str, number: i32) {
        self.fields.push(ProtoField {
            name: name.to_string(),
            number,
            field_type: ProtoFieldType::Message(message_type.to_string()),
            repeated: true,
        });
    }
}

pub struct ServiceBuilder {
    methods: Vec<ProtoMethod>,
}

impl ServiceBuilder {
    pub fn unary(&mut self, name: &str, input: &str, output: &str) {
        self.methods.push(ProtoMethod {
            name: name.to_string(),
            input_type: input.to_string(),
            output_type: output.to_string(),
        });
    }
}
