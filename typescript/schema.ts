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
  | EndTurnParams;
export type MessageChunk = {
  type: "text";
  chunk: string;
};
export type ThreadId = string;
export type TurnId = number;
export type AnyClientResult =
  | StreamMessageChunkResponse
  | ReadTextFileResponse
  | ReadBinaryFileResponse
  | StatResponse
  | GlobSearchResponse
  | EndTurnResponse;
export type StreamMessageChunkResponse = null;
export type FileVersion = number;
export type EndTurnResponse = null;
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
export type ThreadEntry =
  | {
      type: "message";
      chunks: MessageChunk[];
      role: Role;
    }
  | {
      type: "read_file";
      content: string;
      path: string;
    };
export type SendMessageResponse = null;

export interface StreamMessageChunkParams {
  chunk: MessageChunk;
  thread_id: ThreadId;
  turn_id: TurnId;
}
export interface ReadTextFileParams {
  line_limit?: number | null;
  line_offset?: number | null;
  path: string;
  thread_id: ThreadId;
  turn_id: TurnId;
}
export interface ReadBinaryFileParams {
  byte_limit?: number | null;
  byte_offset?: number | null;
  path: string;
  thread_id: ThreadId;
  turn_id: TurnId;
}
export interface StatParams {
  path: string;
  thread_id: ThreadId;
  turn_id: TurnId;
}
export interface GlobSearchParams {
  pattern: string;
  thread_id: ThreadId;
  turn_id: TurnId;
}
export interface EndTurnParams {
  thread_id: ThreadId;
  turn_id: TurnId;
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
  is_directory: boolean;
}
export interface GlobSearchResponse {
  matches: string[];
}
export interface OpenThreadParams {
  thread_id: ThreadId;
}
export interface GetThreadEntriesParams {
  thread_id: ThreadId;
}
export interface SendMessageParams {
  message: Message;
  thread_id: ThreadId;
  turn_id: TurnId;
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
  modified_at: string;
}
export interface CreateThreadResponse {
  thread_id: ThreadId;
}
export interface GetThreadEntriesResponse {
  entries: ThreadEntry[];
}

export interface Client {
  streamMessageChunk?(
    params: StreamMessageChunkParams,
  ): Promise<StreamMessageChunkResponse>;
  readTextFile?(params: ReadTextFileParams): Promise<ReadTextFileResponse>;
  readBinaryFile?(
    params: ReadBinaryFileParams,
  ): Promise<ReadBinaryFileResponse>;
  stat?(params: StatParams): Promise<StatResponse>;
  globSearch?(params: GlobSearchParams): Promise<GlobSearchResponse>;
  endTurn?(params: EndTurnParams): Promise<EndTurnResponse>;
}

export const CLIENT_METHODS = {
  stream_message_chunk: "streamMessageChunk",
  read_text_file: "readTextFile",
  read_binary_file: "readBinaryFile",
  stat: "stat",
  glob_search: "globSearch",
  end_turn: "endTurn",
} as const;

export interface Agent {
  getThreads?(params: GetThreadsParams): Promise<GetThreadsResponse>;
  createThread?(params: CreateThreadParams): Promise<CreateThreadResponse>;
  openThread?(params: OpenThreadParams): Promise<OpenThreadResponse>;
  getThreadEntries?(
    params: GetThreadEntriesParams,
  ): Promise<GetThreadEntriesResponse>;
  sendMessage?(params: SendMessageParams): Promise<SendMessageResponse>;
}

export const AGENT_METHODS = {
  get_threads: "getThreads",
  create_thread: "createThread",
  open_thread: "openThread",
  get_thread_entries: "getThreadEntries",
  send_message: "sendMessage",
} as const;
