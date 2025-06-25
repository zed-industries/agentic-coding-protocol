use std::path::PathBuf;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Serialize)]
pub struct Method {
    pub name: &'static str,
    pub request_type: &'static str,
    pub response_type: &'static str,
}

pub trait AnyRequest: Serialize + Sized {
    type Response: Serialize;
    fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self>;
    fn response_from_method_and_result(method: &str, params: &RawValue) -> Result<Self::Response>;
}

macro_rules! acp_peer {
    (
        $handler_trait_name:ident,
        $request_trait_name:ident,
        $request_enum_name:ident,
        $response_enum_name:ident,
        $method_map_name:ident,
        $(($request_method:ident, $request_name:ident, $response_name:ident)),*
        $(,)?
    ) => {
        #[async_trait(?Send)]
        pub trait $handler_trait_name {
            async fn call(&self, params: $request_enum_name) -> Result<$response_enum_name> {
                match params {
                    $($request_enum_name::$request_name(params) => {
                        let response = self.$request_method(params).await?;
                        Ok($response_enum_name::$response_name(response))
                    }),*
                }
            }

            $(
                async fn $request_method(&self, request: $request_name) -> Result<$response_name>;
            )*
        }

        pub trait $request_trait_name {
            type Response;
            fn into_any(self) -> $request_enum_name;
            fn response_from_any(any: $response_enum_name) -> Option<Self::Response>;
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $request_enum_name {
            $(
                $request_name($request_name),
            )*
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $response_enum_name {
            $(
                $response_name($response_name),
            )*
        }

        impl AnyRequest for $request_enum_name {
            type Response = $response_enum_name;

            fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self> {
                match method {
                    $(
                        stringify!($request_method) => {
                            match serde_json::from_str(params.get()) {
                                Ok(params) => Ok($request_enum_name::$request_name(params)),
                                Err(e) => Err(anyhow!(e.to_string())),
                            }
                        }
                    )*
                    _ => Err(anyhow!("invalid method string {}", method)),
                }
            }

            fn response_from_method_and_result(method: &str, params: &RawValue) -> Result<Self::Response> {
                match method {
                    $(
                        stringify!($request_method) => {
                            match serde_json::from_str(params.get()) {
                                Ok(params) => Ok($response_enum_name::$response_name(params)),
                                Err(e) => Err(anyhow!(e.to_string())),
                            }
                        }
                    )*
                    _ => Err(anyhow!("invalid method string {}", method)),
                }
            }
        }

        impl $request_enum_name {
            pub fn method_name(&self) -> &'static str {
                match self {
                    $(
                        $request_enum_name::$request_name(_) => stringify!($request_method),
                    )*
                }
            }
        }

        pub static $method_map_name: &[Method] = &[
            $(
                Method {
                    name: stringify!($request_method),
                    request_type: stringify!($request_name),
                    response_type: stringify!($response_name),
                },
            )*
        ];

        $(
            impl $request_trait_name for $request_name {
                type Response = $response_name;

                fn into_any(self) -> $request_enum_name {
                    $request_enum_name::$request_name(self)
                }

                fn response_from_any(any: $response_enum_name) -> Option<Self::Response> {
                    match any {
                        $response_enum_name::$response_name(this) => Some(this),
                        _ => None
                    }
                }
            }
        )*
    };
}

acp_peer!(
    Client,
    ClientRequest,
    AnyClientRequest,
    AnyClientResult,
    CLIENT_METHODS,
    (
        stream_message_chunk,
        StreamMessageChunkParams,
        StreamMessageChunkResponse
    ),
    (read_text_file, ReadTextFileParams, ReadTextFileResponse),
    (
        read_binary_file,
        ReadBinaryFileParams,
        ReadBinaryFileResponse
    ),
    (stat, StatParams, StatResponse),
    (glob_search, GlobSearchParams, GlobSearchResponse),
    (end_turn, EndTurnParams, EndTurnResponse),
);

acp_peer!(
    Agent,
    AgentRequest,
    AnyAgentRequest,
    AnyAgentResult,
    AGENT_METHODS,
    (get_threads, GetThreadsParams, GetThreadsResponse),
    (create_thread, CreateThreadParams, CreateThreadResponse),
    (open_thread, OpenThreadParams, OpenThreadResponse),
    (
        get_thread_entries,
        GetThreadEntriesParams,
        GetThreadEntriesResponse
    ),
    (send_message, SendMessageParams, SendMessageResponse),
);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetThreadsParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetThreadsResponse {
    pub threads: Vec<ThreadMetadata>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetThreadEntriesParams {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetThreadEntriesResponse {
    pub entries: Vec<ThreadEntry>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThreadEntry {
    Message {
        #[serde(flatten)]
        message: Message,
    },
    ReadFile {
        path: PathBuf,
        content: String,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub role: Role,
    pub chunks: Vec<MessageChunk>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageChunk {
    Text { chunk: String },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ThreadMetadata {
    pub id: ThreadId,
    pub title: String,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateThreadParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateThreadResponse {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadParams {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadResponse;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct ThreadId(pub String);

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TurnId(pub u64);

impl TurnId {
    pub fn post_inc(&mut self) -> TurnId {
        let id = *self;
        self.0 += 1;
        id
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub message: Message,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EndTurnParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EndTurnResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileVersion(pub u64);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StreamMessageChunkParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub chunk: MessageChunk,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StreamMessageChunkResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadTextFileParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadBinaryFileParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadTextFileResponse {
    pub version: FileVersion,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadBinaryFileResponse {
    pub version: FileVersion,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobSearchParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobSearchResponse {
    pub matches: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StatParams {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StatResponse {
    pub exists: bool,
    pub is_directory: bool,
}
