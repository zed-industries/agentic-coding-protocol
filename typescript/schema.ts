export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest =
  | StreamMessageChunkParams
  | RequestToolCallConfirmationParams
  | PushToolCallParams
  | UpdateToolCallParams;
export type MessageChunk = {
  type: "text";
  chunk: string;
};
export type ThreadId = string;
export type ToolCallConfirmation =
  | {
      description?: string | null;
      type: "edit";
      newText: string;
      oldText: string | null;
      path: string;
    }
  | {
      description?: string | null;
      type: "execute";
      command: string;
      rootCommand: string;
    }
  | {
      description?: string | null;
      type: "mcp";
      serverName: string;
      toolDisplayName: string;
      toolName: string;
    }
  | {
      description?: string | null;
      type: "fetch";
      urls: string[];
    }
  | {
      description: string;
      type: "other";
    };
export type Icon =
  | "fileSearch"
  | "folder"
  | "globe"
  | "hammer"
  | "lightBulb"
  | "pencil"
  | "regex"
  | "terminal";
export type ToolCallContent =
  | {
      type: "markdown";
      markdown: string;
    }
  | {
      type: "diff";
      newText: string;
      oldText: string | null;
      path: string;
    };
export type ToolCallStatus = "running" | "finished" | "error";
export type ToolCallId = number;
export type AnyClientResult =
  | StreamMessageChunkResponse
  | RequestToolCallConfirmationResponse
  | PushToolCallResponse
  | UpdateToolCallResponse;
export type StreamMessageChunkResponse = null;
export type ToolCallConfirmationOutcome =
  | "allow"
  | "alwaysAllow"
  | "alwaysAllowMcpServer"
  | "alwaysAllowTool"
  | "reject";
export type UpdateToolCallResponse = null;
export type AnyAgentRequest =
  | InitializeParams
  | AuthenticateParams
  | CreateThreadParams
  | SendMessageParams;
export type InitializeParams = null;
export type AuthenticateParams = null;
export type CreateThreadParams = null;
export type Role = "user" | "assistant";
export type AnyAgentResult =
  | InitializeResponse
  | AuthenticateResponse
  | CreateThreadResponse
  | SendMessageResponse;
export type AuthenticateResponse = null;
export type SendMessageResponse = null;

export interface StreamMessageChunkParams {
  chunk: MessageChunk;
  threadId: ThreadId;
}
export interface RequestToolCallConfirmationParams {
  confirmation: ToolCallConfirmation;
  icon: Icon;
  label: string;
  threadId: ThreadId;
}
export interface PushToolCallParams {
  icon: Icon;
  label: string;
  threadId: ThreadId;
}
export interface UpdateToolCallParams {
  content: ToolCallContent | null;
  status: ToolCallStatus;
  threadId: ThreadId;
  toolCallId: ToolCallId;
}
export interface RequestToolCallConfirmationResponse {
  id: ToolCallId;
  outcome: ToolCallConfirmationOutcome;
}
export interface PushToolCallResponse {
  id: ToolCallId;
}
export interface SendMessageParams {
  message: Message;
  threadId: ThreadId;
}
export interface Message {
  chunks: MessageChunk[];
  role: Role;
}
export interface InitializeResponse {
  isAuthenticated: boolean;
}
export interface CreateThreadResponse {
  threadId: ThreadId;
}

export interface Client {
  streamMessageChunk(
    params: StreamMessageChunkParams,
  ): Promise<StreamMessageChunkResponse>;
  requestToolCallConfirmation(
    params: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse>;
  pushToolCall(params: PushToolCallParams): Promise<PushToolCallResponse>;
  updateToolCall(params: UpdateToolCallParams): Promise<UpdateToolCallResponse>;
}

export const CLIENT_METHODS = new Set([
  "streamMessageChunk",
  "requestToolCallConfirmation",
  "pushToolCall",
  "updateToolCall",
]);

export interface Agent {
  initialize(params: InitializeParams): Promise<InitializeResponse>;
  authenticate(params: AuthenticateParams): Promise<AuthenticateResponse>;
  createThread(params: CreateThreadParams): Promise<CreateThreadResponse>;
  sendMessage(params: SendMessageParams): Promise<SendMessageResponse>;
}

export const AGENT_METHODS = new Set([
  "initialize",
  "authenticate",
  "createThread",
  "sendMessage",
]);
