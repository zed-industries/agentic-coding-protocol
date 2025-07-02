export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest =
  | StreamMessageChunkParams
  | ReadTextFileParams
  | ReadBinaryFileParams
  | StatParams
  | GlobSearchParams
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
      fileDiff: string;
      fileName: string;
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
export type ToolCallContent = {
  type: "markdown";
  markdown: string;
};
export type ToolCallStatus = "running" | "finished" | "error";
export type ToolCallId = number;
export type AnyClientResult =
  | StreamMessageChunkResponse
  | ReadTextFileResponse
  | ReadBinaryFileResponse
  | StatResponse
  | GlobSearchResponse
  | RequestToolCallConfirmationResponse
  | PushToolCallResponse
  | UpdateToolCallResponse;
export type StreamMessageChunkResponse = null;
export type FileVersion = number;
export type ToolCallConfirmationOutcome =
  | "allow"
  | "alwaysAllow"
  | "alwaysAllowMcpServer"
  | "alwaysAllowTool"
  | "reject";
export type UpdateToolCallResponse = null;
export type AnyAgentRequest =
  | GetThreadsParams
  | CreateThreadParams
  | OpenThreadParams
  | GetThreadEntriesParams
  | SendMessageParams;
export type GetThreadsParams = null;
export type CreateThreadParams = null;
export type Role = "user" | "assistant";
export type AnyAgentResult =
  | GetThreadsResponse
  | CreateThreadResponse
  | OpenThreadResponse
  | GetThreadEntriesResponse
  | SendMessageResponse;
export type OpenThreadResponse = null;
export type ThreadEntry = {
  type: "message";
  chunks: MessageChunk[];
  role: Role;
};
export type SendMessageResponse = null;

export interface StreamMessageChunkParams {
  chunk: MessageChunk;
  threadId: ThreadId;
}
export interface ReadTextFileParams {
  lineLimit?: number | null;
  lineOffset?: number | null;
  path: string;
  threadId: ThreadId;
}
export interface ReadBinaryFileParams {
  byteLimit?: number | null;
  byteOffset?: number | null;
  path: string;
  threadId: ThreadId;
}
export interface StatParams {
  path: string;
  threadId: ThreadId;
}
export interface GlobSearchParams {
  pattern: string;
  threadId: ThreadId;
}
export interface RequestToolCallConfirmationParams {
  confirmation: ToolCallConfirmation;
  displayName: string;
  threadId: ThreadId;
}
export interface PushToolCallParams {
  displayName: string;
  threadId: ThreadId;
}
export interface UpdateToolCallParams {
  content: ToolCallContent | null;
  status: ToolCallStatus;
  threadId: ThreadId;
  toolCallId: ToolCallId;
}
export interface ReadTextFileResponse {
  content: string;
  version: FileVersion;
}
export interface ReadBinaryFileResponse {
  content: string;
  version: FileVersion;
}
export interface StatResponse {
  exists: boolean;
  isDirectory: boolean;
}
export interface GlobSearchResponse {
  matches: string[];
}
export interface RequestToolCallConfirmationResponse {
  id: ToolCallId;
  outcome: ToolCallConfirmationOutcome;
}
export interface PushToolCallResponse {
  id: ToolCallId;
}
export interface OpenThreadParams {
  threadId: ThreadId;
}
export interface GetThreadEntriesParams {
  threadId: ThreadId;
}
export interface SendMessageParams {
  message: Message;
  threadId: ThreadId;
}
export interface Message {
  chunks: MessageChunk[];
  role: Role;
}
export interface GetThreadsResponse {
  threads: ThreadMetadata[];
}
export interface ThreadMetadata {
  title: string;
  id: ThreadId;
  modifiedAt: string;
}
export interface CreateThreadResponse {
  threadId: ThreadId;
}
export interface GetThreadEntriesResponse {
  entries: ThreadEntry[];
}

export interface Client {
  streamMessageChunk(
    params: StreamMessageChunkParams,
  ): Promise<StreamMessageChunkResponse>;
  readTextFile(params: ReadTextFileParams): Promise<ReadTextFileResponse>;
  readBinaryFile(params: ReadBinaryFileParams): Promise<ReadBinaryFileResponse>;
  stat(params: StatParams): Promise<StatResponse>;
  globSearch(params: GlobSearchParams): Promise<GlobSearchResponse>;
  requestToolCallConfirmation(
    params: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse>;
  pushToolCall(params: PushToolCallParams): Promise<PushToolCallResponse>;
  updateToolCall(params: UpdateToolCallParams): Promise<UpdateToolCallResponse>;
}

export const CLIENT_METHODS = new Set([
  "streamMessageChunk",
  "readTextFile",
  "readBinaryFile",
  "stat",
  "globSearch",
  "requestToolCallConfirmation",
  "pushToolCall",
  "updateToolCall",
]);

export interface Agent {
  getThreads(params: GetThreadsParams): Promise<GetThreadsResponse>;
  createThread(params: CreateThreadParams): Promise<CreateThreadResponse>;
  openThread(params: OpenThreadParams): Promise<OpenThreadResponse>;
  getThreadEntries(
    params: GetThreadEntriesParams,
  ): Promise<GetThreadEntriesResponse>;
  sendMessage(params: SendMessageParams): Promise<SendMessageResponse>;
}

export const AGENT_METHODS = new Set([
  "getThreads",
  "createThread",
  "openThread",
  "getThreadEntries",
  "sendMessage",
]);
