// ── Helper: generate a single struct from proto-like field definitions ────────

macro_rules! define_proto_struct {
    ($name:ident {}) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, prost::Message)]
        pub struct $name {}
    };
    ($name:ident { $($fields:tt)+ }) => {
        define_proto_struct!(@build $name [] $($fields)*);
    };
    (@build $name:ident [$($acc:tt)*] string $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(string, tag = $tag)] pub $field: String,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*] uint32 $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(uint32, tag = $tag)] pub $field: u32,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*] int32 $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(int32, tag = $tag)] pub $field: i32,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*] int64 $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(int64, tag = $tag)] pub $field: i64,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*] bool $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(bool, tag = $tag)] pub $field: bool,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*] repeated $msg:ident $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(message, repeated, tag = $tag)] pub $field: Vec<$msg>,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*] repeated_string $field:ident = $tag:literal; $($rest:tt)*) => {
        define_proto_struct!(@build $name [
            $($acc)* #[prost(string, repeated, tag = $tag)] pub $field: Vec<String>,
        ] $($rest)*);
    };
    (@build $name:ident [$($acc:tt)*]) => {
        #[derive(Clone, PartialEq, prost::Message)]
        pub struct $name { $($acc)* }
    };
}

// ── Helper: generate proto text for a single message ─────────────────────────

macro_rules! message_proto_text {
    ($name:ident {}) => {
        concat!("message ", stringify!($name), " {}\n\n")
    };
    ($name:ident { $($fields:tt)+ }) => {
        message_proto_text!(@build $name [] $($fields)*)
    };
    (@build $name:ident [$($acc:tt)*] string $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  string ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*] uint32 $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  uint32 ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*] int32 $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  int32 ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*] int64 $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  int64 ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*] bool $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  bool ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*] repeated $msg:ident $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  repeated ", stringify!($msg), " ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*] repeated_string $field:ident = $tag:literal; $($rest:tt)*) => {
        message_proto_text!(@build $name [
            $($acc)* "  repeated string ", stringify!($field), " = ", $tag, ";\n",
        ] $($rest)*)
    };
    (@build $name:ident [$($acc:tt)*]) => {
        concat!("message ", stringify!($name), " {\n", $($acc)* "}\n\n")
    };
}

// ── Main macro: defines all message structs + messages_proto() ───────────────

macro_rules! proto_messages {
    ($(message $name:ident { $($fields:tt)* })*) => {
        $(
            define_proto_struct!($name { $($fields)* });
        )*

        pub fn messages_proto() -> String {
            let mut s = String::new();
            $(
                s.push_str(message_proto_text!($name { $($fields)* }));
            )*
            s
        }
    };
}

// ── Message definitions (source of truth) ────────────────────────────────────

proto_messages! {
    message LsRequest {}

    message LsResponse {
        repeated TmuxSession sessions = "1";
    }

    message TmuxSession {
        string id = "1";
        string name = "2";
        uint32 windows = "3";
        int64 created = "4";
        bool attached = "5";
    }

    message NewSessionRequest {
        string name = "1";
    }

    message NewSessionResponse {
        string id = "1";
        string name = "2";
        uint32 windows = "3";
        int64 created = "4";
        bool attached = "5";
    }

    message KillSessionRequest {
        string target = "1";
    }

    message KillSessionResponse {}

    message KillWindowRequest {
        string target = "1";
    }

    message KillWindowResponse {}

    message KillPaneRequest {
        string target = "1";
    }

    message KillPaneResponse {}

    message ListWindowsRequest {
        string session = "1";
    }

    message ListWindowsResponse {
        repeated TmuxWindow windows = "1";
    }

    message TmuxWindow {
        string id = "1";
        uint32 index = "2";
        string name = "3";
        uint32 panes = "4";
        bool active = "5";
    }

    message ListPanesRequest {
        string target = "1";
    }

    message ListPanesResponse {
        repeated TmuxPaneMsg panes = "1";
    }

    message TmuxPaneMsg {
        string id = "1";
        uint32 width = "2";
        uint32 height = "3";
        bool active = "4";
        string current_path = "5";
        string current_command = "6";
    }

    message SendKeysRequest {
        string target = "1";
        repeated_string keys = "2";
    }

    message SendKeysResponse {}

    message RenameSessionRequest {
        string target = "1";
        string new_name = "2";
    }

    message RenameSessionResponse {}

    message RenameWindowRequest {
        string target = "1";
        string new_name = "2";
    }

    message RenameWindowResponse {}

    message NewWindowRequest {
        string session = "1";
        string name = "2";
    }

    message NewWindowResponse {
        string id = "1";
        uint32 index = "2";
        string name = "3";
        uint32 panes = "4";
        bool active = "5";
    }

    message SplitWindowRequest {
        string target = "1";
        bool horizontal = "2";
    }

    message SplitWindowResponse {
        string id = "1";
        uint32 width = "2";
        uint32 height = "3";
        bool active = "4";
        string current_path = "5";
        string current_command = "6";
    }

    message CapturePaneRequest {
        string target = "1";
    }

    message CapturePaneResponse {
        string content = "1";
    }

    message CapturePaneWithOptionsRequest {
        string target = "1";
        bool has_start_line = "2";
        int32 start_line = "3";
        bool has_end_line = "4";
        int32 end_line = "5";
        bool escape_sequences = "6";
    }

    message CapturePaneWithOptionsResponse {
        string content = "1";
    }

    message CreateSessionWithWindowsRequest {
        string name = "1";
        repeated_string window_names = "2";
    }

    message CreateSessionWithWindowsResponse {
        string id = "1";
        string name = "2";
        uint32 windows = "3";
        int64 created = "4";
        bool attached = "5";
    }

    message SwapPanesRequest {
        string src = "1";
        string dst = "2";
    }

    message SwapPanesResponse {}

    message MoveWindowRequest {
        string source = "1";
        string destination_session = "2";
    }

    message MoveWindowResponse {}

    message SelectWindowRequest {
        string target = "1";
    }

    message SelectWindowResponse {}

    message SelectPaneRequest {
        string target = "1";
    }

    message SelectPaneResponse {}

    message ResizePaneRequest {
        string target = "1";
        string direction = "2";
        uint32 amount = "3";
    }

    message ResizePaneResponse {}

    message StreamPaneOutputRequest {
        string target = "1";
        uint32 interval_ms = "2";
    }

    message StreamPaneOutputResponse {
        string content = "1";
        int64 timestamp = "2";
    }
}
