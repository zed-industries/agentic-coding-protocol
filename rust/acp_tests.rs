use super::*;

pub struct TestClient;
pub struct TestAgent;

#[async_trait]
impl Client for TestClient {
    async fn list_threads(&self, _request: ListThreadsParams) -> ListThreadsResponse {
        ListThreadsResponse { threads: vec![] }
    }

    async fn open_thread(&self, _request: OpenThreadParams) -> OpenThreadResponse {
        OpenThreadResponse { events: vec![] }
    }
}

#[async_trait]
impl Agent for TestAgent {
    async fn read_file(&self, _request: ReadFileParams) -> ReadFileResponse {
        ReadFileResponse {
            version: FileVersion(0),
            content: "the content".into(),
        }
    }
}
