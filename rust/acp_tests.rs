use super::*;
use anyhow::Result;
use async_trait::async_trait;
use tokio;
use tokio::time::{Duration, timeout};

pub struct TestClient;
pub struct TestAgent;

#[async_trait]
impl Agent for TestAgent {
    async fn list_threads(&self, _request: ListThreadsParams) -> Result<ListThreadsResponse> {
        Ok(ListThreadsResponse { threads: vec![] })
    }

    async fn open_thread(&self, _request: OpenThreadParams) -> Result<OpenThreadResponse> {
        Ok(OpenThreadResponse { events: vec![] })
    }
}

#[async_trait]
impl Client for TestClient {
    async fn read_file(&self, _request: ReadFileParams) -> Result<ReadFileResponse> {
        Ok(ReadFileResponse {
            version: FileVersion(0),
            content: "the content".into(),
        })
    }
}

#[tokio::test]
async fn test_client_agent_communication() {
    let client = TestClient;
    let agent = TestAgent;

    let (client_to_agent_tx, client_to_agent_rx) = async_pipe::pipe();
    let (agent_to_client_tx, agent_to_client_rx) = async_pipe::pipe();

    let (client_connection, client_handle_task, client_io_task) =
        Connection::client_to_agent(client, client_to_agent_tx, agent_to_client_rx);
    let (agent_connection, agent_handle_task, agent_io_task) =
        Connection::agent_to_client(agent, agent_to_client_tx, client_to_agent_rx);

    let _task = tokio::spawn(client_handle_task);
    let _task = tokio::spawn(client_io_task);
    let _task = tokio::spawn(agent_handle_task);
    let _task = tokio::spawn(agent_io_task);

    let response = agent_connection.request(ReadFileParams {
        path: "test.txt".to_string(),
    });
    let response = timeout(Duration::from_secs(2), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.content, "the content");
    assert_eq!(response.version.0, 0);

    let response = client_connection.request(ListThreadsParams);
    let response = timeout(Duration::from_secs(2), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.threads.len(), 0);
}
