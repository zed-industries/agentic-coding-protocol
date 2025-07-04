export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest =
  | StreamAssistantMessageChunkParams
  | RequestToolCallConfirmationParams
  | PushToolCallParams
  | UpdateToolCallParams;
export type AssistantMessageChunk =
  | {
      type: "text";
      chunk: string;
    }
  | {
      type: "thought";
      chunk: string;
    };
export type ToolCallConfirmation =
  | {
      description?: string | null;
      type: "edit";
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
export type Icon =
  | "fileSearch"
  | "folder"
  | "globe"
  | "hammer"
  | "lightBulb"
  | "pencil"
  | "regex"
  | "terminal";
export type ToolCallStatus = "running" | "finished" | "error";
export type ToolCallId = number;
export type AnyClientResult =
  | StreamAssistantMessageChunkResponse
  | RequestToolCallConfirmationResponse
  | PushToolCallResponse
  | UpdateToolCallResponse;
export type StreamAssistantMessageChunkResponse = null;
export type ToolCallConfirmationOutcome =
  | "allow"
  | "alwaysAllow"
  | "alwaysAllowMcpServer"
  | "alwaysAllowTool"
  | "reject"
  | "cancel";
export type UpdateToolCallResponse = null;
export type AnyAgentRequest =
  | InitializeParams
  | AuthenticateParams
  | SendUserMessageParams
  | CancelSendMessageParams;
export type InitializeParams = null;
export type AuthenticateParams = null;
export type UserMessageChunk =
  | {
      type: "text";
      chunk: string;
    }
  | {
      type: "path";
      path: string;
    };
export type CancelSendMessageParams = null;
export type AnyAgentResult =
  | InitializeResponse
  | AuthenticateResponse
  | SendUserMessageResponse
  | CancelSendMessageResponse;
export type AuthenticateResponse = null;
export type SendUserMessageResponse = null;
export type CancelSendMessageResponse = null;

export interface StreamAssistantMessageChunkParams {
  chunk: AssistantMessageChunk;
}
export interface RequestToolCallConfirmationParams {
  confirmation: ToolCallConfirmation;
  content?: ToolCallContent | null;
  icon: Icon;
  label: string;
}
export interface PushToolCallParams {
  content?: ToolCallContent | null;
  icon: Icon;
  label: string;
}
export interface UpdateToolCallParams {
  content: ToolCallContent | null;
  status: ToolCallStatus;
  toolCallId: ToolCallId;
}
export interface RequestToolCallConfirmationResponse {
  id: ToolCallId;
  outcome: ToolCallConfirmationOutcome;
}
export interface PushToolCallResponse {
  id: ToolCallId;
}
export interface SendUserMessageParams {
  message: UserMessage;
}
export interface UserMessage {
  chunks: UserMessageChunk[];
}
export interface InitializeResponse {
  isAuthenticated: boolean;
}

export interface Client {
  streamAssistantMessageChunk(
    params: StreamAssistantMessageChunkParams,
  ): Promise<StreamAssistantMessageChunkResponse>;
  requestToolCallConfirmation(
    params: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse>;
  pushToolCall(params: PushToolCallParams): Promise<PushToolCallResponse>;
  updateToolCall(params: UpdateToolCallParams): Promise<UpdateToolCallResponse>;
}

export const CLIENT_METHODS = new Set([
  "streamAssistantMessageChunk",
  "requestToolCallConfirmation",
  "pushToolCall",
  "updateToolCall",
]);

export interface Agent {
  initialize(params: InitializeParams): Promise<InitializeResponse>;
  authenticate(params: AuthenticateParams): Promise<AuthenticateResponse>;
  sendUserMessage(
    params: SendUserMessageParams,
  ): Promise<SendUserMessageResponse>;
  cancelSendMessage(
    params: CancelSendMessageParams,
  ): Promise<CancelSendMessageResponse>;
}

export const AGENT_METHODS = new Set([
  "initialize",
  "authenticate",
  "sendUserMessage",
  "cancelSendMessage",
]);
