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
    (read_file, ReadFileParams, ReadFileResponse),
    (glob_search, GlobSearchParams, GlobSearchResponse),
);

acp_peer!(
    Agent,
    AgentRequest,
    AnyAgentRequest,
    AnyAgentResult,
    AGENT_METHODS,
    (list_threads, ListThreadsParams, ListThreadsResponse),
    (open_thread, OpenThreadParams, OpenThreadResponse),
);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListThreadsParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListThreadsResponse {
    pub threads: Vec<ThreadMetadata>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ThreadMetadata {
    pub id: ThreadId,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadParams {
    pub thread_id: ThreadId,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadResponse {
    pub events: Vec<ThreadEvent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ThreadId(pub String);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum ThreadEvent {
    UserMessage(Vec<MessageSegment>),
    AgentMessage(Vec<MessageSegment>),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum MessageSegment {
    Text(String),
    Image {
        format: String,
        /// Base64-encoded image data
        content: String,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadFileParams {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileVersion(pub u64);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadFileResponse {
    pub version: FileVersion,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobSearchParams {
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobSearchResponse {
    pub matches: Vec<String>,
}
