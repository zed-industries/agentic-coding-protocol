import { describe, it, expect, beforeEach } from "vitest";
import {
  Agent,
  Client,
  Connection,
  GetThreadsParams,
  GetThreadsResponse,
  OpenThreadParams,
  OpenThreadResponse,
  ReadTextFileParams,
} from "./acp.js";

describe("Connection", () => {
  let clientToAgent: TransformStream;
  let agentToClient: TransformStream;

  beforeEach(() => {
    clientToAgent = new TransformStream();
    agentToClient = new TransformStream();
  });

  it("allows bidirectional communication between client and agent", async () => {
    class TestClient implements Client {
      async readTextFile({ path }: ReadTextFileParams) {
        return {
          content: `Contents of ${path}`,
          version: 1,
        };
      }
    }

    class TestAgent implements Agent {
      async getThreads() {
        return {
          threads: [
            { id: "thread-1", title: "First Thread", modified_at: "" },
            { id: "thread-2", title: "Second Thread", modified_at: "" },
          ],
        };
      }

      async openThread(_params: { thread_id: string }) {
        return null;
      }
    }

    const agentConnection = Connection.clientToAgent(
      (_connection) => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (_connection) => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    const fileContent = await clientConnection.readTextFile!({
      thread_id: "thread-1",
      turn_id: 0,
      path: "/test/file.ts",
    });
    expect(fileContent).toEqual({
      content: "Contents of /test/file.ts",
      version: 1,
    });

    const threads = await agentConnection.getThreads!(null);
    expect(threads).toEqual({
      threads: [
        { id: "thread-1", title: "First Thread", modified_at: "" },
        { id: "thread-2", title: "Second Thread", modified_at: "" },
      ],
    });

    const threadData = await agentConnection.openThread!({
      thread_id: "thread-1",
    });
    expect(threadData).toBeNull();
  });

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    class TestClient implements Client {
      async readTextFile(_params: ReadTextFileParams): Promise<never> {
        throw new Error("File not found");
      }
    }

    // Create agent that throws errors
    class TestAgent implements Agent {
      async getThreads(_: GetThreadsParams): Promise<GetThreadsResponse> {
        throw new Error("Failed to list threads");
      }
      async openThread(_: OpenThreadParams): Promise<OpenThreadResponse> {
        throw new Error("Failed to open thread");
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (_connection) => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (_connection) => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test error handling in client->agent direction
    await expect(
      clientConnection.readTextFile!({
        thread_id: "thread-1",
        turn_id: 0,
        path: "/missing.ts",
      }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(agentConnection.getThreads!(null)).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    let callCount = 0;

    // Create client with delayed responses
    class TestClient implements Client {
      async readTextFile({ path }: ReadTextFileParams) {
        await new Promise((resolve) => setTimeout(resolve, 40));
        return {
          content: `Delayed content of ${path}`,
          version: Date.now(),
        };
      }
    }

    // Create agent with delayed responses
    class TestAgent implements Agent {
      async getThreads() {
        callCount++;
        await new Promise((resolve) => setTimeout(resolve, 50));
        return {
          threads: [
            {
              id: `thread-${callCount}`,
              title: `Thread ${callCount}`,
              modified_at: "",
            },
          ],
        };
      }
      async openThread(_params: { thread_id: string }) {
        await new Promise((resolve) => setTimeout(resolve, 30));
        return null;
      }
    }

    const agentConnection = Connection.clientToAgent(
      (_connection) => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (_connection) => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send multiple concurrent requests
    const promises = [
      clientConnection.readTextFile!({
        thread_id: "test-thread",
        turn_id: 0,
        path: "/file1.ts",
      }),
      clientConnection.readTextFile!({
        thread_id: "test-thread",
        turn_id: 0,
        path: "/file2.ts",
      }),
      agentConnection.getThreads!(null),
      agentConnection.openThread!({ thread_id: "test-thread" }),
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

    class TestClient implements Client {
      async readTextFile({ path }: ReadTextFileParams) {
        messageLog.push(`readTextFile called with ${path}`);
        return { content: "", version: 0 };
      }
    }

    class TestAgent implements Agent {
      async getThreads() {
        messageLog.push("getThreads called");
        return { threads: [] };
      }
      async openThread({ thread_id }: OpenThreadParams) {
        messageLog.push(`openThread called with ${thread_id}`);
        return null;
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (_connection) => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (_connection) => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send requests in specific order
    await clientConnection.readTextFile!({
      thread_id: "thread-x",
      turn_id: 0,
      path: "/first.ts",
    });
    await agentConnection.getThreads!(null);
    await clientConnection.readTextFile!({
      thread_id: "thread-x",
      turn_id: 0,
      path: "/second.ts",
    });
    await agentConnection.openThread!({ thread_id: "thread-x" });

    // Verify order
    expect(messageLog).toEqual([
      "readTextFile called with /first.ts",
      "getThreads called",
      "readTextFile called with /second.ts",
      "openThread called with thread-x",
    ]);
  });
});
