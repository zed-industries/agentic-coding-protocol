export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest =
  | StreamMessageChunkParams
  | ReadTextFileParams
  | RequestToolCallParams
  | ReadBinaryFileParams
  | StatParams
  | GlobSearchParams;
export type MessageChunk = {
  type: "text";
  chunk: string;
};
export type ThreadId = string;
export type AnyClientResult =
  | StreamMessageChunkResponse
  | ReadTextFileResponse
  | RequestToolCallResponse
  | ReadBinaryFileResponse
  | StatResponse
  | GlobSearchResponse;
export type StreamMessageChunkResponse = null;
export type FileVersion = number;
export type RequestToolCallResponse =
  | {
      type: "allowed";
      id: ToolCallId;
    }
  | {
      type: "rejected";
    };
export type ToolCallId = number;
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
export interface RequestToolCallParams {
  description: string;
  threadId: ThreadId;
  toolName: string;
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
  requestToolCall(
    params: RequestToolCallParams,
  ): Promise<RequestToolCallResponse>;
  readBinaryFile(params: ReadBinaryFileParams): Promise<ReadBinaryFileResponse>;
  stat(params: StatParams): Promise<StatResponse>;
  globSearch(params: GlobSearchParams): Promise<GlobSearchResponse>;
}

export const CLIENT_METHODS = new Set([
  "streamMessageChunk",
  "readTextFile",
  "requestToolCall",
  "readBinaryFile",
  "stat",
  "globSearch",
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
