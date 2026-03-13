#[derive(Clone, Copy, PartialEq, Eq, Hash, prost::Message)]
pub struct LsRequest {}

#[derive(Clone, PartialEq, prost::Message)]
pub struct LsResponse {
    #[prost(message, repeated, tag = "1")]
    pub sessions: Vec<TmuxSession>,
}

#[derive(Clone, PartialEq, Eq, Hash, prost::Message)]
pub struct TmuxSession {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(uint32, tag = "2")]
    pub windows: u32,
    #[prost(string, tag = "3")]
    pub created: String,
    #[prost(bool, tag = "4")]
    pub attached: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, prost::Message)]
pub struct NewSessionRequest {
    #[prost(string, tag = "1")]
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, prost::Message)]
pub struct NewSessionResponse {
    #[prost(string, tag = "1")]
    pub name: String,
}
