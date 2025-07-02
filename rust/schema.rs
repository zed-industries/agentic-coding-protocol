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
        $(($request_method:ident, $request_method_string:expr, $request_name:ident, $response_name:ident)),*
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
                        $request_method_string => {
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
                        $request_method_string => {
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
                        $request_enum_name::$request_name(_) => $request_method_string,
                    )*
                }
            }
        }

        pub static $method_map_name: &[Method] = &[
            $(
                Method {
                    name: $request_method_string,
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
        "streamMessageChunk",
        StreamMessageChunkParams,
        StreamMessageChunkResponse
    ),
    (
        read_text_file,
        "readTextFile",
        ReadTextFileParams,
        ReadTextFileResponse
    ),
    (
        read_binary_file,
        "readBinaryFile",
        ReadBinaryFileParams,
        ReadBinaryFileResponse
    ),
    (stat, "stat", StatParams, StatResponse),
    (
        glob_search,
        "globSearch",
        GlobSearchParams,
        GlobSearchResponse
    ),
    (
        request_tool_call_confirmation,
        "requestToolCallConfirmation",
        RequestToolCallConfirmationParams,
        RequestToolCallConfirmationResponse
    ),
    (
        push_tool_call,
        "pushToolCall",
        PushToolCallParams,
        PushToolCallResponse
    ),
    (
        update_tool_call,
        "updateToolCall",
        UpdateToolCallParams,
        UpdateToolCallResponse
    ),
);

acp_peer!(
    Agent,
    AgentRequest,
    AnyAgentRequest,
    AnyAgentResult,
    AGENT_METHODS,
    (
        initialize,
        "initialize",
        InitializeParams,
        InitializeResponse
    ),
    (
        authenticate,
        "authenticate",
        AuthenticateParams,
        AuthenticateResponse
    ),
    (
        get_threads,
        "getThreads",
        GetThreadsParams,
        GetThreadsResponse
    ),
    (
        create_thread,
        "createThread",
        CreateThreadParams,
        CreateThreadResponse
    ),
    (
        open_thread,
        "openThread",
        OpenThreadParams,
        OpenThreadResponse
    ),
    (
        get_thread_entries,
        "getThreadEntries",
        GetThreadEntriesParams,
        GetThreadEntriesResponse
    ),
    (
        send_message,
        "sendMessage",
        SendMessageParams,
        SendMessageResponse
    )
);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub is_authenticated: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetThreadsParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetThreadsResponse {
    pub threads: Vec<ThreadMetadata>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetThreadEntriesParams {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetThreadEntriesResponse {
    pub entries: Vec<ThreadEntry>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ThreadEntry {
    Message {
        #[serde(flatten)]
        message: Message,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: Role,
    pub chunks: Vec<MessageChunk>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MessageChunk {
    Text { chunk: String },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMetadata {
    pub id: ThreadId,
    pub title: String,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadResponse {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpenThreadParams {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpenThreadResponse;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ThreadId(pub String);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageParams {
    pub thread_id: ThreadId,
    pub message: Message,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileVersion(pub u64);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessageChunkParams {
    pub thread_id: ThreadId,
    pub chunk: MessageChunk,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessageChunkResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileParams {
    pub thread_id: ThreadId,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadBinaryFileParams {
    pub thread_id: ThreadId,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileResponse {
    pub version: FileVersion,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadBinaryFileResponse {
    pub version: FileVersion,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobSearchParams {
    pub thread_id: ThreadId,
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobSearchResponse {
    pub matches: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatParams {
    pub thread_id: ThreadId,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatResponse {
    pub exists: bool,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestToolCallConfirmationParams {
    pub thread_id: ThreadId,
    pub label: String,
    pub icon: Icon,
    pub confirmation: ToolCallConfirmation,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Icon {
    FileSearch,
    Folder,
    Globe,
    Hammer,
    LightBulb,
    Pencil,
    Regex,
    Terminal,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallConfirmation {
    #[serde(rename_all = "camelCase")]
    Edit {
        file_name: String,
        file_diff: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Execute {
        command: String,
        root_command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Mcp {
        server_name: String,
        tool_name: String,
        tool_display_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Fetch {
        urls: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Other { description: String },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct RequestToolCallConfirmationResponse {
    pub id: ToolCallId,
    pub outcome: ToolCallConfirmationOutcome,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallConfirmationOutcome {
    Allow,
    AlwaysAllow,
    AlwaysAllowMcpServer,
    AlwaysAllowTool,
    Reject,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PushToolCallParams {
    pub thread_id: ThreadId,
    pub label: String,
    pub icon: Icon,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct PushToolCallResponse {
    pub id: ToolCallId,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallId(pub u64);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateToolCallParams {
    pub thread_id: ThreadId,
    pub tool_call_id: ToolCallId,
    pub status: ToolCallStatus,
    pub content: Option<ToolCallContent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateToolCallResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallStatus {
    Running,
    Finished,
    Error,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallContent {
    Markdown { markdown: String },
    // Diff,
    // Snippet {
    //     path: PathBuf,
    //     start_line: u32,
    //     end_line: u32,
    // },
}
