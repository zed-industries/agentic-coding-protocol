import { describe, it, expect, beforeEach } from "vitest";
import {
  Agent,
  AuthenticateParams,
  AuthenticateResponse,
  Client,
  Connection,
  CreateThreadParams,
  CreateThreadResponse,
  InitializeParams,
  InitializeResponse,
  PushToolCallParams,
  PushToolCallResponse,
  RequestToolCallConfirmationParams,
  RequestToolCallConfirmationResponse,
  SendUserMessageParams,
  SendUserMessageResponse,
  StreamAssistantMessageChunkParams,
  StreamAssistantMessageChunkResponse,
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

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    class TestClient extends StubClient {
      async pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
        throw new Error("Tool call failed");
      }
    }

    // Create agent that throws errors
    class TestAgent extends StubAgent {
      async createThreads(
        _: CreateThreadParams,
      ): Promise<CreateThreadResponse> {
        throw new Error("Failed to create thread");
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
      clientConnection.pushToolCall({
        threadId: "thread-1",
        label: "/missing.ts",
        icon: "fileSearch",
      }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(agentConnection.createThread!(null)).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    let callCount = 0;

    // Create client with delayed responses
    class TestClient extends StubClient {
      toolCall: number = 0;

      async pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
        this.toolCall++;
        const id = this.toolCall;
        await new Promise((resolve) => setTimeout(resolve, 40));
        return { id };
      }
    }

    // Create agent with delayed responses
    class TestAgent extends StubAgent {
      async createThread(_: CreateThreadParams): Promise<CreateThreadResponse> {
        callCount++;
        const threadId = `thread-${callCount}`;
        await new Promise((resolve) => setTimeout(resolve, 50));
        return { threadId };
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
      agentConnection.createThread(null),
      clientConnection.pushToolCall({
        threadId: "test-thread",
        label: "Tool Call 1",
        icon: "fileSearch",
      }),
      clientConnection.pushToolCall({
        threadId: "test-thread",
        label: "Tool Call 2",
        icon: "fileSearch",
      }),
      agentConnection.createThread(null),
      clientConnection.pushToolCall({
        threadId: "test-thread",
        label: "Tool Call 3",
        icon: "fileSearch",
      }),
    ];

    const results = await Promise.all(promises);

    // Verify all requests completed successfully
    expect(results[0]).toHaveProperty("threadId", "thread-1");
    expect(results[1]).toHaveProperty("id", 1);
    expect(results[2]).toHaveProperty("id", 2);
    expect(results[3]).toHaveProperty("threadId", "thread-2");
    expect(results[4]).toHaveProperty("id", 3);

    expect(callCount).toBe(2);
  });

  it("handles message ordering correctly", async () => {
    const messageLog: string[] = [];

    class TestClient extends StubClient {
      async pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
        messageLog.push("pushToolCall called");
        return { id: 0 };
      }
      async updateToolCall(
        _: UpdateToolCallParams,
      ): Promise<UpdateToolCallResponse> {
        messageLog.push("updateToolCall called");
        return null;
      }
    }

    class TestAgent extends StubAgent {
      async initialize(_: InitializeParams): Promise<InitializeResponse> {
        messageLog.push("initialize called");
        return { isAuthenticated: true };
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
    await agentConnection.initialize!(null);
    let { id } = await clientConnection.pushToolCall({
      threadId: "thread-x",
      icon: "folder",
      label: "Folder",
    });
    await clientConnection.updateToolCall({
      content: {
        type: "markdown",
        markdown: "Markdown",
      },
      status: "finished",
      threadId: "thread-x",
      toolCallId: id,
    });

    // Verify order
    expect(messageLog).toEqual([
      "initialize called",
      "pushToolCall called",
      "updateToolCall called",
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
  createThread(_: CreateThreadParams): Promise<CreateThreadResponse> {
    throw new Error("Method not implemented.");
  }
  sendUserMessage(_: SendUserMessageParams): Promise<SendUserMessageResponse> {
    throw new Error("Method not implemented.");
  }
}

class StubClient implements Client {
  constructor(private agent: Agent) {}
  streamAssistantMessageChunk(
    _: StreamAssistantMessageChunkParams,
  ): Promise<StreamAssistantMessageChunkResponse> {
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
