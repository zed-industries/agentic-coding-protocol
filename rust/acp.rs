#[cfg(test)]
mod acp_tests;

use async_trait::async_trait;
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

macro_rules! request {
    (
        $trait_name:ident,
        $type_name:ident,
        $result_type_name:ident,
        $method_map_name:ident,
        $(($request_method:ident, $request_name:ident, $response_name:ident)),*
        $(,)?
    ) => {
        #[async_trait]
        pub trait $trait_name {
            $(
                async fn $request_method(&self, request: $request_name) -> $response_name;
            )*
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $type_name {
            $(
                $request_name($request_name),
            )*
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $result_type_name {
            $(
                $response_name($response_name),
            )*
        }

        $(impl Request for $request_name {
            const METHOD: &'static str = stringify!($request_method);
            type Response = $response_name;
        })*

        pub static $method_map_name: &[Method] = &[
            $(
                Method {
                    name: stringify!($request_method),
                    request_type: stringify!($request_name),
                    response_type: stringify!($response_name),
                },
            )*
        ];
    };
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Message {
    ClientRequest(ClientRequest),
    ClientResult(ClientResult),
    AgentRequest(AgentRequest),
    AgentResult(AgentResult),
}

request!(
    Client,
    ClientRequest,
    ClientResult,
    CLIENT_METHODS,
    (list_threads, ListThreadsParams, ListThreadsResponse),
    (open_thread, OpenThreadParams, OpenThreadResponse),
);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ListThreadsParams;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ListThreadsResponse {
    threads: Vec<ThreadMetadata>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ThreadMetadata {
    id: ThreadId,
    title: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadParams {
    thread_id: ThreadId,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenThreadResponse {
    events: Vec<ThreadEvent>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ThreadId(String);

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
    path: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct FileVersion(u64);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReadFileResponse {
    version: FileVersion,
    content: String,
}

request!(
    Agent,
    AgentRequest,
    AgentResult,
    AGENT_METHODS,
    (read_file, ReadFileParams, ReadFileResponse),
);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Point {
    pub row: u32,
    pub column: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Range {
    pub start: Point,
    pub end: Point,
}
