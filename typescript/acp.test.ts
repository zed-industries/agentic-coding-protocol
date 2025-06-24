import { describe, it, expect, beforeEach } from "vitest";
import { Agent, Client, Connection } from "./acp.js";

describe("Connection", () => {
  let clientToAgent: TransformStream;
  let agentToClient: TransformStream;

  beforeEach(() => {
    clientToAgent = new TransformStream();
    agentToClient = new TransformStream();
  });

  it("allows bidirectional communication between client and agent", async () => {
    const client: Client = {
      readFile: async ({ path }) => ({
        content: `Contents of ${path}`,
        version: 1,
      }),
    };

    const agent: Agent = {
      listThreads: async () => ({
        threads: [
          { id: "thread-1", title: "First Thread" },
          { id: "thread-2", title: "Second Thread" },
        ],
      }),
      openThread: async ({ thread_id }) => ({
        events: [
          { UserMessage: [{ Text: `Opening thread ${thread_id}` }] },
          { AgentMessage: [{ Text: "Thread opened successfully" }] },
        ],
      }),
    };

    const agentConnection = Connection.clientToAgent(
      client,
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      agent,
      agentToClient.writable,
      clientToAgent.readable,
    );

    const fileContent = await clientConnection.readFile({
      path: "/test/file.ts",
    });
    expect(fileContent).toEqual({
      content: "Contents of /test/file.ts",
      version: 1,
    });

    const threads = await agentConnection.listThreads(null);
    expect(threads).toEqual({
      threads: [
        { id: "thread-1", title: "First Thread" },
        { id: "thread-2", title: "Second Thread" },
      ],
    });

    const threadData = await agentConnection.openThread({
      thread_id: "thread-1",
    });
    expect(threadData).toEqual({
      events: [
        { UserMessage: [{ Text: "Opening thread thread-1" }] },
        { AgentMessage: [{ Text: "Thread opened successfully" }] },
      ],
    });
  });

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    const client: Client = {
      readFile: async () => {
        throw new Error("File not found");
      },
    };

    // Create agent that throws errors
    const agent: Agent = {
      listThreads: async () => {
        throw new Error("Failed to list threads");
      },
      openThread: async () => {
        throw new Error("Failed to open thread");
      },
    };

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      client,
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      agent,
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test error handling in client->agent direction
    await expect(
      clientConnection.readFile({ path: "/missing.ts" }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(agentConnection.listThreads(null)).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    let callCount = 0;

    // Create client with delayed responses
    const client: Client = {
      readFile: async ({ path }) => {
        await new Promise((resolve) => setTimeout(resolve, 40));
        return {
          content: `Delayed content of ${path}`,
          version: Date.now(),
        };
      },
    };

    // Create agent with delayed responses
    const agent: Agent = {
      listThreads: async () => {
        callCount++;
        await new Promise((resolve) => setTimeout(resolve, 50));
        return {
          threads: [
            { id: `thread-${callCount}`, title: `Thread ${callCount}` },
          ],
        };
      },
      openThread: async ({ thread_id }) => {
        await new Promise((resolve) => setTimeout(resolve, 30));
        return {
          events: [{ UserMessage: [{ Text: `Opened ${thread_id}` }] }],
        };
      },
    };

    const agentConnection = Connection.clientToAgent(
      client,
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      agent,
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send multiple concurrent requests
    const promises = [
      clientConnection.readFile({ path: "/file1.ts" }),
      clientConnection.readFile({ path: "/file2.ts" }),
      agentConnection.listThreads(null),
      agentConnection.openThread({ thread_id: "test-thread" }),
      agentConnection.listThreads(null),
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
    expect(results[3]).toHaveProperty("events");
    expect(results[4]).toHaveProperty("threads");

    // Verify that concurrent listThreads calls were handled
    expect(callCount).toBe(2);
  });

  it("handles message ordering correctly", async () => {
    const messageLog: string[] = [];

    const client: Client = {
      readFile: async ({ path }) => {
        messageLog.push(`readFile called with ${path}`);
        return { content: "", version: 0 };
      },
    };

    const agent: Agent = {
      listThreads: async () => {
        messageLog.push("listThreads called");
        return { threads: [] };
      },
      openThread: async ({ thread_id }) => {
        messageLog.push(`openThread called with ${thread_id}`);
        return { events: [] };
      },
    };

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      client,
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      agent,
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send requests in specific order
    await clientConnection.readFile({ path: "/first.ts" });
    await agentConnection.listThreads(null);
    await clientConnection.readFile({ path: "/second.ts" });
    await agentConnection.openThread({ thread_id: "thread-x" });

    // Verify order
    expect(messageLog).toEqual([
      "readFile called with /first.ts",
      "listThreads called",
      "readFile called with /second.ts",
      "openThread called with thread-x",
    ]);
  });
});
