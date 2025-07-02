import { describe, it, expect, beforeEach } from "vitest";
import {
  Agent,
  AuthenticateParams,
  AuthenticateResponse,
  Client,
  Connection,
  CreateThreadParams,
  CreateThreadResponse,
  GetThreadEntriesParams,
  GetThreadEntriesResponse,
  GetThreadsParams,
  GetThreadsResponse,
  GlobSearchParams,
  GlobSearchResponse,
  InitializeParams,
  InitializeResponse,
  OpenThreadParams,
  OpenThreadResponse,
  PushToolCallParams,
  PushToolCallResponse,
  ReadBinaryFileParams,
  ReadBinaryFileResponse,
  ReadTextFileParams,
  ReadTextFileResponse,
  RequestToolCallConfirmationParams,
  RequestToolCallConfirmationResponse,
  SendMessageParams,
  SendMessageResponse,
  StatParams,
  StatResponse,
  StreamMessageChunkParams,
  StreamMessageChunkResponse,
  UpdateToolCallParams,
  UpdateToolCallResponse,
} from "./acp.js";

describe("Connection", () => {
  let clientToAgent: TransformStream;
  let agentToClient: TransformStream;

  beforeEach(() => {
    clientToAgent = new TransformStream();
    agentToClient = new TransformStream();
  });

  it("allows bidirectional communication between client and agent", async () => {
    class TestClient extends StubClient {
      async readTextFile({ path }: ReadTextFileParams) {
        return {
          content: `Contents of ${path}`,
          version: 1,
        };
      }
    }

    class TestAgent extends StubAgent {
      async getThreads(_: GetThreadsParams): Promise<GetThreadsResponse> {
        return {
          threads: [
            { id: "thread-1", title: "First Thread", modifiedAt: "" },
            { id: "thread-2", title: "Second Thread", modifiedAt: "" },
          ],
        };
      }

      async openThread(_params: { threadId: string }) {
        return null;
      }
    }

    const agentConnection = Connection.clientToAgent(
      (agent) => new TestClient(agent),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    const fileContent = await clientConnection.readTextFile({
      threadId: "thread-1",
      path: "/test/file.ts",
    });
    expect(fileContent).toEqual({
      content: "Contents of /test/file.ts",
      version: 1,
    });

    const threads = await agentConnection.getThreads!(null);
    expect(threads).toEqual({
      threads: [
        { id: "thread-1", title: "First Thread", modifiedAt: "" },
        { id: "thread-2", title: "Second Thread", modifiedAt: "" },
      ],
    });

    const threadData = await agentConnection.openThread!({
      threadId: "thread-1",
    });
    expect(threadData).toBeNull();
  });

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    class TestClient extends StubClient {
      async readTextFile(_params: ReadTextFileParams): Promise<never> {
        throw new Error("File not found");
      }
    }

    // Create agent that throws errors
    class TestAgent extends StubAgent {
      async getThreads(_: GetThreadsParams): Promise<GetThreadsResponse> {
        throw new Error("Failed to list threads");
      }
      async openThread(_: OpenThreadParams): Promise<OpenThreadResponse> {
        throw new Error("Failed to open thread");
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (agent) => new TestClient(agent),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test error handling in client->agent direction
    await expect(
      clientConnection.readTextFile({
        threadId: "thread-1",
        path: "/missing.ts",
      }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(agentConnection.getThreads!(null)).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    let callCount = 0;

    // Create client with delayed responses
    class TestClient extends StubClient {
      async readTextFile({ path }: ReadTextFileParams) {
        await new Promise((resolve) => setTimeout(resolve, 40));
        return {
          content: `Delayed content of ${path}`,
          version: Date.now(),
        };
      }
    }

    // Create agent with delayed responses
    class TestAgent extends StubAgent {
      async getThreads() {
        callCount++;
        await new Promise((resolve) => setTimeout(resolve, 50));
        return {
          threads: [
            {
              id: `thread-${callCount}`,
              title: `Thread ${callCount}`,
              modifiedAt: "",
            },
          ],
        };
      }
      async openThread(_params: { threadId: string }) {
        await new Promise((resolve) => setTimeout(resolve, 30));
        return null;
      }
    }

    const agentConnection = Connection.clientToAgent(
      (a) => new TestClient(a),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send multiple concurrent requests
    const promises = [
      clientConnection.readTextFile({
        threadId: "test-thread",
        path: "/file1.ts",
      }),
      clientConnection.readTextFile({
        threadId: "test-thread",
        path: "/file2.ts",
      }),
      agentConnection.getThreads!(null),
      agentConnection.openThread!({ threadId: "test-thread" }),
      agentConnection.getThreads!(null),
    ];

    const results = await Promise.all(promises);

    // Verify all requests completed successfully
    expect(results[0]).toHaveProperty(
      "content",
      "Delayed content of /file1.ts",
    );
    expect(results[1]).toHaveProperty(
      "content",
      "Delayed content of /file2.ts",
    );
    expect(results[2]).toHaveProperty("threads");
    expect(results[3]).toBeNull();
    expect(results[4]).toHaveProperty("threads");

    // Verify that concurrent getThreads calls were handled
    expect(callCount).toBe(2);
  });

  it("handles message ordering correctly", async () => {
    const messageLog: string[] = [];

    class TestClient extends StubClient {
      async readTextFile({ path }: ReadTextFileParams) {
        messageLog.push(`readTextFile called with ${path}`);
        return { content: "", version: 0 };
      }
    }

    class TestAgent extends StubAgent {
      async getThreads() {
        messageLog.push("getThreads called");
        return { threads: [] };
      }
      async openThread({ threadId }: OpenThreadParams) {
        messageLog.push(`openThread called with ${threadId}`);
        return null;
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (client) => new TestClient(client),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send requests in specific order
    await clientConnection.readTextFile({
      threadId: "thread-x",
      path: "/first.ts",
    });
    await agentConnection.getThreads!(null);
    await clientConnection.readTextFile({
      threadId: "thread-x",
      path: "/second.ts",
    });
    await agentConnection.openThread!({ threadId: "thread-x" });

    // Verify order
    expect(messageLog).toEqual([
      "readTextFile called with /first.ts",
      "getThreads called",
      "readTextFile called with /second.ts",
      "openThread called with thread-x",
    ]);
  });
});

class StubAgent implements Agent {
  constructor(private client: Client) {}
  initialize(_: InitializeParams): Promise<InitializeResponse> {
    throw new Error("Method not implemented.");
  }
  authenticate(_: AuthenticateParams): Promise<AuthenticateResponse> {
    throw new Error("Method not implemented.");
  }
  getThreads(_: GetThreadsParams): Promise<GetThreadsResponse> {
    throw new Error("Method not implemented.");
  }
  createThread(_: CreateThreadParams): Promise<CreateThreadResponse> {
    throw new Error("Method not implemented.");
  }
  openThread(_: OpenThreadParams): Promise<OpenThreadResponse> {
    throw new Error("Method not implemented.");
  }
  getThreadEntries(
    _: GetThreadEntriesParams,
  ): Promise<GetThreadEntriesResponse> {
    throw new Error("Method not implemented.");
  }
  sendMessage(_: SendMessageParams): Promise<SendMessageResponse> {
    throw new Error("Method not implemented.");
  }
}

class StubClient implements Client {
  constructor(private agent: Agent) {}
  streamMessageChunk(
    _: StreamMessageChunkParams,
  ): Promise<StreamMessageChunkResponse> {
    throw new Error("Method not implemented.");
  }
  readTextFile(_: ReadTextFileParams): Promise<ReadTextFileResponse> {
    throw new Error("Method not implemented.");
  }
  readBinaryFile(_: ReadBinaryFileParams): Promise<ReadBinaryFileResponse> {
    throw new Error("Method not implemented.");
  }
  stat(_: StatParams): Promise<StatResponse> {
    throw new Error("Method not implemented.");
  }
  globSearch(_: GlobSearchParams): Promise<GlobSearchResponse> {
    throw new Error("Method not implemented.");
  }
  requestToolCallConfirmation(
    _: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse> {
    throw new Error("Method not implemented.");
  }
  pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
    throw new Error("Method not implemented.");
  }
  updateToolCall(_: UpdateToolCallParams): Promise<UpdateToolCallResponse> {
    throw new Error("Method not implemented.");
  }
}
