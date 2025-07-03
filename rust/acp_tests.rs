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
    async fn initialize(&self, _request: InitializeParams) -> Result<InitializeResponse> {
        Ok(InitializeResponse {
            is_authenticated: true,
        })
    }

    async fn authenticate(&self, _request: AuthenticateParams) -> Result<AuthenticateResponse> {
        Ok(AuthenticateResponse)
    }

    async fn create_thread(&self, _request: CreateThreadParams) -> Result<CreateThreadResponse> {
        Ok(CreateThreadResponse {
            thread_id: ThreadId("test-thread".into()),
        })
    }

    async fn send_message(&self, _request: SendMessageParams) -> Result<SendMessageResponse> {
        Ok(SendMessageResponse)
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

    async fn request_tool_call_confirmation(
        &self,
        _request: RequestToolCallConfirmationParams,
    ) -> Result<RequestToolCallConfirmationResponse> {
        Ok(RequestToolCallConfirmationResponse {
            id: ToolCallId(0),
            outcome: ToolCallConfirmationOutcome::Allow,
        })
    }

    async fn push_tool_call(&self, _request: PushToolCallParams) -> Result<PushToolCallResponse> {
        Ok(PushToolCallResponse { id: ToolCallId(0) })
    }

    async fn update_tool_call(
        &self,
        _request: UpdateToolCallParams,
    ) -> Result<UpdateToolCallResponse> {
        Ok(UpdateToolCallResponse)
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

            let response = agent_connection.request(PushToolCallParams {
                thread_id: ThreadId("0".into()),
                label: "test".into(),
                icon: Icon::FileSearch,
                content: None,
            });
            let response = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(response.id, ToolCallId(0));

            let response = client_connection.request(CreateThreadParams);
            let response = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(response.thread_id, ThreadId("test-thread".into()));
        })
        .await
}
