export type AgentCodingProtocol =
  | ClientRequest
  | ClientResult
  | AgentRequest
  | AgentResult;
export type ClientRequest = ReadFileParams;
export type ClientResult = ReadFileResponse;
export type FileVersion = number;
export type AgentRequest = ListThreadsParams | OpenThreadParams;
export type ListThreadsParams = null;
export type ThreadId = string;
export type AgentResult = ListThreadsResponse | OpenThreadResponse;
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
export interface ReadFileResponse {
  content: string;
  version: FileVersion;
}
export interface OpenThreadParams {
  thread_id: ThreadId;
}
export interface ListThreadsResponse {
  threads: ThreadMetadata[];
}
export interface ThreadMetadata {
  title: string;
  id: ThreadId;
}
export interface OpenThreadResponse {
  events: ThreadEvent[];
}

export interface Client {
  readFile(params: ReadFileParams): Promise<ReadFileResponse>;
}

export const CLIENT_METHODS = {
  read_file: "readFile",
} as const;

export interface Agent {
  listThreads(params: ListThreadsParams): Promise<ListThreadsResponse>;
  openThread(params: OpenThreadParams): Promise<OpenThreadResponse>;
}

export const AGENT_METHODS = {
  list_threads: "listThreads",
  open_thread: "openThread",
} as const;
