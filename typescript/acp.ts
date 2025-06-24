import { Agent, AGENT_METHODS, Client, CLIENT_METHODS } from "./schema";

export * from "./schema";

type PendingResponse = {
  resolve: (response: any) => void;
  reject: (error: any) => void;
};

export class Connection {
  #pendingResponses: Map<number, PendingResponse> = new Map();
  #nextRequestId: number = 0;
  #delegate: Object;
  #delegateMethods: Record<string, string>;
  #peerInput: WritableStream;
  #writeQueue: Promise<void> = Promise.resolve();

  constructor(
    delegate: Object,
    delegateMethods: Record<string, string>,
    peerMethods: Record<string, string>,
    peerInput: WritableStream,
    peerOutput: ReadableStream,
  ) {
    this.#delegate = delegate;
    this.#delegateMethods = delegateMethods;
    this.#peerInput = peerInput;

    for (const [protoMethodName, jsMethodName] of Object.entries(peerMethods)) {
      this[jsMethodName] = (params: unknown) => {
        return this.#sendRequest(protoMethodName, params);
      };
    }

    this.#receive(peerOutput);
  }

  static clientToAgent(
    client: Client,
    input: WritableStream,
    output: ReadableStream,
  ): Agent {
    return new Connection(
      client,
      CLIENT_METHODS,
      AGENT_METHODS,
      input,
      output,
    ) as any as Agent;
  }

  static agentToClient(
    agent: Agent,
    input: WritableStream,
    output: ReadableStream,
  ): Client {
    return new Connection(
      agent,
      AGENT_METHODS,
      CLIENT_METHODS,
      input,
      output,
    ) as any as Client;
  }

  async #receive(output: ReadableStream) {
    let content = "";
    for await (const chunk of output) {
      content += chunk;
      const lines = content.split("\n");
      content = lines.pop() || "";

      for (const line of lines) {
        const trimmedLine = line.trim();
        if (trimmedLine) {
          const message = JSON.parse(trimmedLine);
          if (message.method) {
            const methodName = this.#delegateMethods[message.method];
            if (
              methodName &&
              typeof this.#delegate[methodName] === "function"
            ) {
              try {
                const result = await this.#delegate[methodName](message.params);
                this.#writeJSON({ id: message.id, result });
              } catch (error) {
                this.#writeJSON({
                  id: message.id,
                  error: {
                    code: error.code ?? 500,
                    message: error.message,
                  },
                });
              }
            } else {
              this.#writeJSON({
                id: message.id,
                error: { code: 404, message: "Method Not Found" },
              });
            }
          } else {
            const pendingResponse = this.#pendingResponses.get(message.id);
            if (pendingResponse) {
              if (message.result) {
                pendingResponse.resolve(message.result);
              } else if (message.error) {
                pendingResponse.reject(message.result);
              }
              this.#pendingResponses.delete(message.id);
            }
          }
        }
      }
    }
  }

  async #sendRequest(method: string, params: unknown): Promise<unknown> {
    const id = this.#nextRequestId++;
    const responsePromise = new Promise((resolve, reject) => {
      this.#pendingResponses.set(id, { resolve, reject });
    });
    await this.#writeJSON({ id, method, params });
    return responsePromise;
  }

  async #writeJSON(json: unknown) {
    const content = JSON.stringify(json) + "\n";
    this.#writeQueue = this.#writeQueue
      .then(async () => {
        const writer = this.#peerInput.getWriter();
        try {
          await writer.write(content);
        } finally {
          writer.releaseLock();
        }
      })
      .catch(() => {
        // Continue processing writes on error
      });
    return this.#writeQueue;
  }
}
