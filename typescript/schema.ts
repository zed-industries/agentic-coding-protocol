export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest = ReadFileParams | GlobSearchParams;
export type AnyClientResult = ReadFileResponse | GlobSearchResponse;
export type FileVersion = number;
export type AnyAgentRequest = ListThreadsParams | OpenThreadParams;
export type ListThreadsParams = null;
export type ThreadId = string;
export type AnyAgentResult = ListThreadsResponse | OpenThreadResponse;
export type ThreadEvent =
  | {
      UserMessage: MessageSegment[];
    }
  | {
      AgentMessage: MessageSegment[];
    };
export type MessageSegment =
  | {
      Text: string;
    }
  | {
      Image: {
        format: string;
        /**
         * Base64-encoded image data
         */
        content: string;
      };
    };

export interface ReadFileParams {
  path: string;
}
export interface GlobSearchParams {
  pattern: string;
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
export interface ListThreadsResponse {
  threads: ThreadMetadata[];
}
export interface ThreadMetadata {
  title: string;
  created_at: string;
  id: ThreadId;
}
export interface OpenThreadResponse {
  events: ThreadEvent[];
}

export interface Client {
  readFile?(params: ReadFileParams): Promise<ReadFileResponse>;
  globSearch?(params: GlobSearchParams): Promise<GlobSearchResponse>;
}

export const CLIENT_METHODS = {
  read_file: "readFile",
  glob_search: "globSearch",
} as const;

export interface Agent {
  listThreads?(params: ListThreadsParams): Promise<ListThreadsResponse>;
  openThread?(params: OpenThreadParams): Promise<OpenThreadResponse>;
}

export const AGENT_METHODS = {
  list_threads: "listThreads",
  open_thread: "openThread",
} as const;
