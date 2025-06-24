use anyhow::{Result, anyhow};
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
        $trait_name:ident,
        $type_name:ident,
        $result_type_name:ident,
        $method_map_name:ident,
        $(($request_method:ident, $request_name:ident, $response_name:ident)),*
        $(,)?
    ) => {
        #[async_trait]
        pub trait $trait_name {
            /// Call a method on the client by name.
            async fn call(&self, method_name: Box<str>, params: Box<str>) -> Result<Box<str>> {
                match method_name.as_ref() {
                    $(stringify!($request_method) => {
                        // todo! move json parsing to background io loop
                        let request = serde_json::from_str::<$request_name>(&params)?;
                        let response = self.$request_method(request).await?;
                        Ok(serde_json::to_string(&response)?.into())
                    }),*
                    _ => Err(anyhow!("method {:?} not found", method_name)),
                }
            }

            $(
                async fn $request_method(&self, request: $request_name) -> Result<$response_name>;
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

acp_peer!(
    Client,
    ClientRequest,
    ClientResult,
    CLIENT_METHODS,
    (read_file, ReadFileParams, ReadFileResponse),
);

acp_peer!(
    Agent,
    AgentRequest,
    AgentResult,
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
