#[cfg(test)]
mod acp_tests;
mod schema;

use anyhow::{Result, anyhow};
use futures::{
    AsyncBufReadExt as _, AsyncRead, AsyncWrite, AsyncWriteExt as _, FutureExt as _,
    StreamExt as _,
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::LocalBoxFuture,
    io::BufReader,
    select_biased,
};
use parking_lot::Mutex;
pub use schema::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering::SeqCst},
    },
};

/// A connection to a separate agent process over the ACP protocol.
pub struct AgentConnection(Connection<AnyClientRequest, AnyAgentRequest>);

/// A connection to a separate client process over the ACP protocol.
pub struct ClientConnection(Connection<AnyAgentRequest, AnyClientRequest>);

impl AgentConnection {
    /// Connect to an agent process, handling any incoming requests
    /// using the given handler.
    pub fn connect_to_agent<H: 'static + Client>(
        handler: H,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> (
        Self,
        impl Future<Output = ()>,
        impl Future<Output = Result<()>>,
    ) {
        let handler = Arc::new(handler);
        let (connection, handler_task, io_task) = Connection::new(
            Box::new(move |request| {
                let handler = handler.clone();
                async move { handler.call(request).await }.boxed_local()
            }),
            outgoing_bytes,
            incoming_bytes,
        );
        (Self(connection), handler_task, io_task)
    }

    /// Send a request to the agent and wait for a response.
    pub fn request<R: AgentRequest>(&self, params: R) -> impl Future<Output = Result<R::Response>> {
        let params = params.into_any();
        let result = self.0.request(params.method_name(), params);
        async move {
            let result = result.await?;
            R::response_from_any(result).ok_or_else(|| anyhow!("wrong response type"))
        }
    }
}

impl ClientConnection {
    pub fn connect_to_client<H: 'static + Agent>(
        handler: H,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> (
        Self,
        impl Future<Output = ()>,
        impl Future<Output = Result<()>>,
    ) {
        let handler = Arc::new(handler);
        let (connection, handler_task, io_task) = Connection::new(
            Box::new(move |request| {
                let handler = handler.clone();
                async move { handler.call(request).await }.boxed_local()
            }),
            outgoing_bytes,
            incoming_bytes,
        );
        (Self(connection), handler_task, io_task)
    }

    pub fn request<R: ClientRequest>(
        &self,
        params: R,
    ) -> impl Future<Output = Result<R::Response>> {
        let params = params.into_any();
        let result = self.0.request(params.method_name(), params);
        async move {
            let result = result.await?;
            R::response_from_any(result).ok_or_else(|| anyhow!("wrong response type"))
        }
    }
}

struct Connection<In, Out>
where
    In: Request,
    Out: Request,
{
    outgoing_tx: UnboundedSender<AnyMessage<Out, In::Response>>,
    response_senders: ResponseSenders<Out::Response>,
    next_id: AtomicI32,
}

type ResponseSenders<T> = Arc<Mutex<HashMap<i32, oneshot::Sender<Result<T>>>>>;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum AnyMessage<Req, Resp> {
    Request {
        id: i32,
        method: Box<str>,
        params: Req,
    },
    OkResponse {
        id: i32,
        result: Resp,
    },
    ErrorResponse {
        id: i32,
        error: String,
    },
}

impl<In, Out> Connection<In, Out>
where
    In: Request,
    Out: Request,
{
    fn new(
        request_handler: Box<dyn 'static + Fn(In) -> LocalBoxFuture<'static, Result<In::Response>>>,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> (
        Self,
        impl Future<Output = ()>,
        impl Future<Output = Result<()>>,
    ) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded();
        let (incoming_tx, incoming_rx) = mpsc::unbounded();
        let this = Self {
            response_senders: ResponseSenders::default(),
            outgoing_tx: outgoing_tx.clone(),
            next_id: AtomicI32::new(0),
        };
        let handler_task = Self::handle_incoming(
            outgoing_tx,
            incoming_rx,
            request_handler,
            this.response_senders.clone(),
        );
        let io_task = Self::handle_io(outgoing_rx, incoming_tx, outgoing_bytes, incoming_bytes);
        (this, handler_task, io_task)
    }

    fn request(
        &self,
        method: &'static str,
        params: Out,
    ) -> impl Future<Output = Result<Out::Response>> {
        let (tx, rx) = oneshot::channel();
        let id = self.next_id.fetch_add(1, SeqCst);
        self.response_senders.lock().insert(id, tx);
        self.outgoing_tx
            .unbounded_send(AnyMessage::Request {
                id,
                method: method.into(),
                params: params,
            })
            .ok();
        async move { rx.await? }
    }

    async fn handle_io(
        mut outgoing_rx: UnboundedReceiver<AnyMessage<Out, In::Response>>,
        incoming_tx: UnboundedSender<AnyMessage<In, Out::Response>>,
        mut outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> Result<()>
    where
        In: Serialize + DeserializeOwned,
        Out: Serialize + DeserializeOwned,
    {
        let mut output_reader = BufReader::new(incoming_bytes);
        let mut outgoing_line = Vec::new();
        let mut incoming_line = String::new();
        loop {
            select_biased! {
                message = outgoing_rx.next() => {
                    if let Some(message) = message {
                        outgoing_line.clear();
                        serde_json::to_writer(&mut outgoing_line, &message)?;
                        log::trace!("send: {}", String::from_utf8_lossy(&outgoing_line));
                        outgoing_line.push(b'\n');
                        outgoing_bytes.write_all(&outgoing_line).await.ok();
                    } else {
                        break;
                    }
                }
                bytes_read = output_reader.read_line(&mut incoming_line).fuse() => {
                    if bytes_read? == 0 {
                        break
                    }
                    log::trace!("recv: {}", &incoming_line);

                    if let Ok(message) = serde_json::from_str(&incoming_line) {
                        if incoming_tx.unbounded_send(message).is_err() {
                            break
                        }
                    }
                    incoming_line.clear();
                }
            }
        }
        Ok(())
    }

    async fn handle_incoming(
        outgoing_tx: UnboundedSender<AnyMessage<Out, In::Response>>,
        mut incoming_rx: UnboundedReceiver<AnyMessage<In, Out::Response>>,
        incoming_handler: Box<
            dyn 'static + Fn(In) -> LocalBoxFuture<'static, Result<In::Response>>,
        >,
        response_senders: ResponseSenders<Out::Response>,
    ) {
        while let Some(message) = incoming_rx.next().await {
            match message {
                AnyMessage::Request { id, params, .. } => {
                    let result = incoming_handler(params).await;
                    match result {
                        Ok(result) => {
                            outgoing_tx
                                .unbounded_send(AnyMessage::OkResponse { id, result })
                                .ok();
                        }
                        Err(error) => {
                            outgoing_tx
                                .unbounded_send(AnyMessage::ErrorResponse {
                                    id,
                                    error: error.to_string(),
                                })
                                .ok();
                        }
                    }
                }
                AnyMessage::OkResponse { id, result } => {
                    let sender = response_senders.lock().remove(&id);
                    if let Some(sender) = sender {
                        sender.send(Ok(result)).ok();
                    }
                }
                AnyMessage::ErrorResponse { id, error } => {
                    let sender = response_senders.lock().remove(&id);
                    if let Some(sender) = sender {
                        sender.send(Err(anyhow!("{}", error))).ok();
                    }
                }
            }
        }
    }
}
