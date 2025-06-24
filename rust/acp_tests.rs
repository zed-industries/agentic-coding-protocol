use super::*;
use anyhow::Result;
use async_trait::async_trait;
use futures::channel::mpsc;
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

    let (client_to_agent_tx, client_to_agent_rx) = mpsc::unbounded();
    let (agent_to_client_tx, agent_to_client_rx) = mpsc::unbounded();

    let (client_connection, client_handle_task) =
        Connection::client_to_agent(client, client_to_agent_tx, agent_to_client_rx);
    let (agent_connection, agent_handle_task) =
        Connection::agent_to_client(agent, agent_to_client_tx, client_to_agent_rx);

    let client_handle = tokio::spawn(client_handle_task);
    let agent_handle = tokio::spawn(agent_handle_task);

    let response = agent_connection.request(ReadFileParams {
        path: "test.txt".to_string(),
    });
    let response = timeout(Duration::from_secs(2), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.content, "the content");
    assert_eq!(response.version.0, 0);

    // Test client requesting thread list (client connection sends request, agent connection handles it)
    let response = client_connection.request(ListThreadsParams);
    let response = timeout(Duration::from_secs(2), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.threads.len(), 0);

    // Clean up
    client_handle.abort();
    agent_handle.abort();
}
