import { Agent, AGENT_METHODS, Client, CLIENT_METHODS } from "./schema.js";

export * from "./schema.js";

type PendingResponse = {
  resolve: (response: unknown) => void;
  reject: (error: unknown) => void;
};

type AnyMessage = AnyRequest | AnyResponse;

type AnyRequest = {
  id: number;
  method: string;
  params: unknown;
};

type AnyResponse = { id: number } & Result<unknown>;

type Result<T> =
  | {
      result: T;
    }
  | {
      error: {
        code: number;
        message: string;
      };
    };

export class Connection<D, P> {
  #pendingResponses: Map<number, PendingResponse> = new Map();
  #nextRequestId: number = 0;
  #delegate: D;
  #delegateMethods: Record<string, keyof D>;
  #peerInput: WritableStream<Uint8Array>;
  #writeQueue: Promise<void> = Promise.resolve();
  #textEncoder: TextEncoder;

  constructor(
    delegate: (peer: P) => D,
    delegateMethods: Record<string, keyof D>,
    peerMethods: Record<string, keyof P>,
    peerInput: WritableStream<Uint8Array>,
    peerOutput: ReadableStream<Uint8Array>,
  ) {
    this.#delegateMethods = delegateMethods;
    this.#peerInput = peerInput;
    this.#textEncoder = new TextEncoder();

    const peer = this as unknown as Record<
      keyof P,
      (params: unknown) => Promise<unknown>
    >;

    for (const [protoMethodName, jsMethodName] of Object.entries(peerMethods)) {
      peer[jsMethodName] = (params: unknown) => {
        return this.#sendRequest(protoMethodName, params);
      };
    }

    this.#delegate = delegate(this as unknown as P);
    this.#receive(peerOutput);
  }

  static clientToAgent(
    client: (agent: Agent) => Client,
    input: WritableStream<Uint8Array>,
    output: ReadableStream<Uint8Array>,
  ): Agent {
    return new Connection<Client, Agent>(
      client,
      CLIENT_METHODS,
      AGENT_METHODS,
      input,
      output,
    ) as unknown as Agent;
  }

  static agentToClient(
    agent: (client: Client) => Agent,
    input: WritableStream,
    output: ReadableStream,
  ): Client {
    return new Connection<Agent, Client>(
      agent,
      AGENT_METHODS,
      CLIENT_METHODS,
      input,
      output,
    ) as unknown as Client;
  }

  async #receive(output: ReadableStream<Uint8Array>) {
    let content = "";
    const decoder = new TextDecoder();
    for await (const chunk of output) {
      content += decoder.decode(chunk, { stream: true });
      const lines = content.split("\n");
      content = lines.pop() || "";

      for (const line of lines) {
        const trimmedLine = line.trim();

        if (trimmedLine) {
          const message = JSON.parse(trimmedLine);
          this.#processMessage(message);
        }
      }
    }
  }

  async #processMessage(message: AnyMessage) {
    if ("method" in message) {
      let response = await this.#tryCallDelegateMethod(
        message.method,
        message.params,
      );

      await this.#sendMessage({
        id: message.id,
        ...response,
      });
    } else {
      this.#handleResponse(message);
    }
  }

  async #tryCallDelegateMethod(
    method: string,
    params: unknown,
  ): Promise<Result<unknown>> {
    const methodName = this.#delegateMethods[method];

    if (!methodName || typeof this.#delegate[methodName] !== "function") {
      return {
        error: { code: 404, message: "Method Not Found" },
      };
    }

    try {
      const result = await this.#delegate[methodName](params);
      return { result };
    } catch (error: unknown) {
      let code = 500;
      let errMessage = "Unknown Error";

      if (error && typeof error === "object") {
        if ("code" in error && typeof error.code === "number") {
          code = error.code;
        }
        if ("message" in error && typeof error.message === "string") {
          errMessage = error.message;
        }
      }

      return {
        error: { code, message: errMessage },
      };
    }
  }

  #handleResponse(response: AnyResponse) {
    const pendingResponse = this.#pendingResponses.get(response.id);
    if (pendingResponse) {
      if ("result" in response) {
        pendingResponse.resolve(response.result);
      } else if ("error" in response) {
        pendingResponse.reject(response.error);
      }
      this.#pendingResponses.delete(response.id);
    }
  }

  async #sendRequest(method: string, params: unknown): Promise<unknown> {
    const id = this.#nextRequestId++;
    const responsePromise = new Promise((resolve, reject) => {
      this.#pendingResponses.set(id, { resolve, reject });
    });
    await this.#sendMessage({ id, method, params });
    return responsePromise;
  }

  async #sendMessage(json: AnyMessage) {
    const content = JSON.stringify(json) + "\n";
    this.#writeQueue = this.#writeQueue
      .then(async () => {
        const writer = this.#peerInput.getWriter();
        try {
          await writer.write(this.#textEncoder.encode(content));
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
