export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest =
  | ReadFileParams
  | GlobSearchParams
  | EndTurnParams;
export type ThreadId = string;
export type TurnId = number;
export type AnyClientResult =
  | ReadFileResponse
  | GlobSearchResponse
  | EndTurnResponse;
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
export type MessageChunk = {
  type: "text";
  chunk: string;
};
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

export interface ReadFileParams {
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
export interface ReadFileResponse {
  content: string;
  version: FileVersion;
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
export interface SendMessageResponse {
  turn_id: TurnId;
}

export interface Client {
  readFile?(params: ReadFileParams): Promise<ReadFileResponse>;
  globSearch?(params: GlobSearchParams): Promise<GlobSearchResponse>;
  endTurn?(params: EndTurnParams): Promise<EndTurnResponse>;
}

export const CLIENT_METHODS = {
  read_file: "readFile",
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
