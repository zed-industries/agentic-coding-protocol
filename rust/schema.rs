use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub trait Request: Serialize + DeserializeOwned {
    const METHOD: &'static str;
    type Response: Serialize + DeserializeOwned;
}

pub trait Notification: Serialize + DeserializeOwned {
    const METHOD: &'static str;
}

#[derive(Serialize)]
pub struct Method {
    pub name: &'static str,
    pub request_type: &'static str,
    pub response_type: &'static str,
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

        pub trait $request_trait_name: Request {
            fn into_any(self) -> $request_enum_name;
            fn response_from_any(any: $response_enum_name) -> Option<Self::Response>;
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
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

        impl $request_enum_name {
            pub fn method_name(&self) -> &'static str {
                match self {
                    $(
                        $request_enum_name::$request_name(_) => stringify!($request_method),
                    )*
                }
            }
        }

        impl Request for $request_enum_name {
            const METHOD: &'static str = "";
            type Response = $response_enum_name;
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

            impl Request for $request_name {
                const METHOD: &'static str = stringify!($request_method);
                type Response = $response_name;
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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ListThreadsParams;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ListThreadsResponse {
    pub threads: Vec<ThreadMetadata>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ThreadMetadata {
    pub id: ThreadId,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadParams {
    pub thread_id: ThreadId,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadResponse {
    pub events: Vec<ThreadEvent>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ThreadId(pub String);

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum ThreadEvent {
    UserMessage(Vec<MessageSegment>),
    AgentMessage(Vec<MessageSegment>),
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum MessageSegment {
    Text(String),
    Image {
        format: String,
        /// Base64-encoded image data
        content: String,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReadFileParams {
    pub path: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct FileVersion(pub u64);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReadFileResponse {
    pub version: FileVersion,
    pub content: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GlobSearchParams {
    pub pattern: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GlobSearchResponse {
    pub matches: Vec<String>,
}
