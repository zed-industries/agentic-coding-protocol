use super::*;
use anyhow::Result;
use async_trait::async_trait;
use tokio;
use tokio::task::LocalSet;
use tokio::time::{Duration, timeout};

pub struct TestClient;
pub struct TestAgent;

#[async_trait(?Send)]
impl Agent for TestAgent {
    async fn get_threads(&self, _request: GetThreadsParams) -> Result<GetThreadsResponse> {
        Ok(GetThreadsResponse { threads: vec![] })
    }

    async fn open_thread(&self, _request: OpenThreadParams) -> Result<OpenThreadResponse> {
        Ok(OpenThreadResponse)
    }

    async fn create_thread(&self, _request: CreateThreadParams) -> Result<CreateThreadResponse> {
        Ok(CreateThreadResponse {
            thread_id: ThreadId("test-thread".into()),
        })
    }

    async fn send_message(&self, _request: SendMessageParams) -> Result<SendMessageResponse> {
        Ok(SendMessageResponse { turn_id: TurnId(0) })
    }

    async fn get_thread_entries(
        &self,
        _request: GetThreadEntriesParams,
    ) -> Result<GetThreadEntriesResponse> {
        Ok(GetThreadEntriesResponse { entries: vec![] })
    }
}

#[async_trait(?Send)]
impl Client for TestClient {
    async fn stream_message_chunk(
        &self,
        _request: StreamMessageChunkParams,
    ) -> Result<StreamMessageChunkResponse> {
        Ok(StreamMessageChunkResponse {})
    }

    async fn read_file(&self, _request: ReadFileParams) -> Result<ReadFileResponse> {
        Ok(ReadFileResponse {
            version: FileVersion(0),
            content: "the content".into(),
        })
    }

    async fn glob_search(&self, _request: GlobSearchParams) -> Result<GlobSearchResponse> {
        Ok(GlobSearchResponse { matches: vec![] })
    }

    async fn end_turn(&self, _request: EndTurnParams) -> Result<EndTurnResponse> {
        Ok(EndTurnResponse {})
    }
}

#[tokio::test]
async fn test_client_agent_communication() {
    env_logger::init();

    let local = LocalSet::new();
    local
        .run_until(async move {
            let client = TestClient;
            let agent = TestAgent;

            let (client_to_agent_tx, client_to_agent_rx) = async_pipe::pipe();
            let (agent_to_client_tx, agent_to_client_rx) = async_pipe::pipe();

            let (client_connection, client_handle_task, client_io_task) =
                AgentConnection::connect_to_agent(client, client_to_agent_tx, agent_to_client_rx);
            let (agent_connection, agent_handle_task, agent_io_task) =
                ClientConnection::connect_to_client(agent, agent_to_client_tx, client_to_agent_rx);

            let _task = tokio::task::spawn_local(client_handle_task);
            let _task = tokio::task::spawn_local(agent_handle_task);
            let _task = tokio::spawn(client_io_task);
            let _task = tokio::spawn(agent_io_task);

            let response = agent_connection.request(ReadFileParams {
                thread_id: ThreadId("0".into()),
                turn_id: TurnId(0),
                path: "test.txt".into(),
            });
            let response = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(response.content, "the content");
            assert_eq!(response.version.0, 0);

            let response = client_connection.request(GetThreadsParams);
            let response = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(response.threads.len(), 0);
        })
        .await
}
