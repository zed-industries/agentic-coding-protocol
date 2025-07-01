# Agent Communication Protocol

This document describes the wire format for the Agent Communication Protocol (ACP), a JSON-based protocol for communication between a development environment (client) and a coding agent (agent).

## Transport

The protocol is transport-agnostic, but it assumes a reliable, ordered, and bidirectional stream-based transport. Messages are sent as newline-delimited JSON objects. Each line is a self-contained JSON message.

## Message Structure

All messages are JSON objects. There are two types of messages: **Requests** and **Responses**.

### Requests

A request is a call to a method on the remote peer.

- `id`: An integer that is unique for each request initiated by a peer.
- `method`: A string containing the name of the method to be invoked.
- `params`: A structured value holding the parameters for the method.

Example Request:
```json
{"id":0,"method":"getThreads","params":null}
```

### Responses

A response is sent by the peer that received a request.

- `id`: The same `id` as the corresponding request.
- `result`: The value returned by the method. This field is required on success.
- `error`: An error object if the method call failed. This field is required on failure.

A response MUST contain either `result` or `error`, but not both.

#### Success Response

Example:
```json
{"id":0,"result":{"threads":[]}}
```

#### Error Response

The `error` object has the following members:
- `code`: An integer indicating the error type.
- `message`: A string providing a short description of the error.

Example:
```json
{"id":1,"error":{"code":-32601,"message":"Method not found - 'nonExistentMethod'"}}
```

## Protocol Methods

The protocol defines methods that can be called by the agent and methods that can be called by the client.

### Agent-initiated Methods (Agent -> Client)

These are requests sent from the agent to the client.

- **`getThreads`**: Fetches a list of all available conversation threads.
  - `params`: `null`
  - `result`: An object with a `threads` array. Each item in the array is a `ThreadMetadata` object with `id`, `title`, and `modifiedAt`.

- **`createThread`**: Creates a new conversation thread.
  - `params`: `null`
  - `result`: An object with a `threadId`.

- **`openThread`**: Opens a specific thread.
  - `params`: An object with a `threadId`.
  - `result`: `null`

- **`getThreadEntries`**: Retrieves the entries (messages, file reads) for a thread.
  - `params`: An object with a `threadId`.
  - `result`: An object with an `entries` array. See `ThreadEntry` in `schema.json` for the structure of entries.

- **`sendMessage`**: Sends a message to a thread.
  - `params`: An object with `threadId` and a `message` object. The `message` object contains the `role` ("user" or "assistant") and an array of `chunks`.
  - `result`: `null`

### Client-initiated Methods (Client -> Agent)

These are requests sent from the client to the agent.

- **`streamMessageChunk`**: Streams a part of a message to the agent.
  - `params`: An object with `threadId` and a `chunk` object.
  - `result`: `null`

- **`readTextFile`**: Requests to read a text file.
  - `params`: An object with `threadId`, `path`, and optional `lineOffset` and `lineLimit`.
  - `result`: An object with `version` and `content` (string).

- **`readBinaryFile`**: Requests to read a binary file.
  - `params`: An object with `threadId`, `path`, and optional `byteOffset` and `byteLimit`.
  - `result`: An object with `version` and `content` (base64 encoded string).

- **`stat`**: Retrieves metadata for a file or directory.
  - `params`: An object with `threadId` and `path`.
  - `result`: An object with `exists` (boolean) and `isDirectory` (boolean).

- **`globSearch`**: Performs a file search using a glob pattern.
  - `params`: An object with `threadId` and `pattern`.
  - `result`: An object with a `matches` array of strings (paths).

## Data Structures

This section provides detailed information on the data structures used in the protocol.

### `ThreadId`

A `ThreadId` is a string that uniquely identifies a conversation thread.

- **Type**: `string`

### `ThreadMetadata`

`ThreadMetadata` contains information about a thread.

- **`id`**: The thread's unique `ThreadId`.
- **`title`**: A string representing the title of the thread.
- **`modifiedAt`**: An RFC 3339 timestamp indicating when the thread was last modified.

Example:
```json
{
  "id": "thread-123",
  "title": "My Conversation",
  "modifiedAt": "2024-07-30T12:00:00Z"
}
```

### `ThreadEntry`

A `ThreadEntry` represents a single event or message within a thread. It can be one of two types: `message` or `readFile`.

#### Message Entry

A `message` entry represents a message from either the user or the assistant.

- **`type`**: The string `"message"`.
- **`role`**: The role of the sender, either `"user"` or `"assistant"`.
- **`chunks`**: An array of `MessageChunk` objects that make up the message content.

Example:
```json
{
  "type": "message",
  "role": "user",
  "chunks": [
    {
      "type": "text",
      "chunk": "Hello, world!"
    }
  ]
}
```

#### ReadFile Entry

A `readFile` entry logs that a file was read.

- **`type`**: The string `"readFile"`.
- **`path`**: The path of the file that was read.
- **`content`**: The content of the file as a string.

Example:
```json
{
  "type": "readFile",
  "path": "/path/to/file.txt",
  "content": "File content goes here."
}
```

### `Message`

A `Message` object represents a complete message in a thread.

- **`role`**: The role of the sender (`"user"` or `"assistant"`).
- **`chunks`**: An array of `MessageChunk` objects.

Example:
```json
{
  "role": "assistant",
  "chunks": [
    {
      "type": "text",
      "chunk": "This is a response."
    }
  ]
}
```

### `MessageChunk`

A `MessageChunk` is a part of a message. Currently, only text chunks are supported.

- **`type`**: The type of the chunk, currently `"text"`.
- **`chunk`**: The string content of the chunk.

Example:
```json
{
  "type": "text",
  "chunk": "This is a piece of the message."
}
```

### `Role`

The `Role` determines the sender of a message.

- **Type**: `string`
- **Values**: `"user"`, `"assistant"`

### `FileVersion`

A `FileVersion` is an integer that represents the version of a file. This can be used to detect concurrent modifications.

- **Type**: `integer` (unsigned 64-bit)
- **Minimum**: `0`
