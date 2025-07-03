# Agentic Coding Protocol

The Agentic Coding Protocol (ACP) is a protocol that standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

The protocol is still under heavy development, and we aim to standardize it as
we get confidence in the design by implementing it in various settings.

## Overview

The protocol is newline-delimited JSON sent over `stdin`/`stdout`. When a code editor wants to start a session with an agent, it boots it as a sub-process (inheriting any environment variables) and sends an `initialize` request to get the state of the world.

If authentication is required, it can send `authenticate` to allow the agent to perform any authentication actions (like an Oauth flow).

Once the agent is ready, the client can send `sendUserMessage` requests with content from the user. The agent sends `streamAssistantMessageChunk` and related tool call messages to update the UI while handling the user's message, and finally responds when there will be no more output.

## Details

The schema is defined in [schema.rs](./rust/schema.rs), and a type-script definition is generated to [schema.ts](./typescript/schema.ts).

This repo also contains interoperable implementations of the protocol for both Typescript and Rust.
