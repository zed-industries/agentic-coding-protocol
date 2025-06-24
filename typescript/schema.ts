export type AgentCodingProtocol = ClientRequest | ClientResult | AgentRequest | AgentResult;
export type ClientRequest = ListThreadsParams | OpenThreadParams;
export type ListThreadsParams = null;
export type ThreadId = string;
export type ClientResult = ListThreadsResponse | OpenThreadResponse;
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
export type AgentRequest = ReadFileParams;
export type AgentResult = ReadFileResponse;
export type FileVersion = number;

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
export interface ReadFileParams {
  path: string;
}
export interface ReadFileResponse {
  content: string;
  version: FileVersion;
}

export interface Client {
  listThreads(params: ListThreadsParams): Promise<ListThreadsResponse>;
  openThread(params: OpenThreadParams): Promise<OpenThreadResponse>;
}

export const CLIENT_METHODS = {
  "list_threads": "listThreads",
  "open_thread": "openThread",
};

export interface Agent {
  readFile(params: ReadFileParams): Promise<ReadFileResponse>;
}

export const AGENT_METHODS = {
  "read_file": "readFile",
};
