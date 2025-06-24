#[cfg(test)]
mod acp_tests;
mod schema;

use anyhow::{Result, anyhow};
use futures::{
    AsyncBufReadExt as _, AsyncRead, AsyncWrite, AsyncWriteExt as _, FutureExt as _,
    StreamExt as _,
    channel::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::BoxFuture,
    io::BufReader,
    select_biased,
};
use parking_lot::Mutex;
pub use schema::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering::SeqCst},
    },
};

pub struct Connection {
    input_tx: UnboundedSender<Box<str>>,
    state: Arc<Mutex<ConnectionState>>,
    next_id: AtomicI32,
}

struct ConnectionState {
    response_senders: HashMap<i32, oneshot::Sender<Result<Box<str>>>>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum AnyMessage {
    Request {
        id: i32,
        method: Box<str>,
        params: Box<str>,
    },
    OkResponse {
        id: i32,
        result: Box<str>,
    },
    ErrorResponse {
        id: i32,
        error: String,
    },
}

type RequestHandler =
    Box<dyn 'static + Send + Fn(Box<str>, Box<str>) -> BoxFuture<'static, Result<Box<str>>>>;

impl Connection {
    pub fn client_to_agent<C>(
        client: C,
        input_bytes: impl Unpin + AsyncWrite,
        output_bytes: impl Unpin + AsyncRead,
    ) -> (
        Self,
        impl Future<Output = ()>,
        impl Future<Output = Result<()>>,
    )
    where
        C: 'static + Send + Sync + Client,
    {
        let client = Arc::new(client);
        Self::new(
            Box::new(move |method, params| {
                let client = client.clone();
                async move { client.call(method, params).await }.boxed()
            }),
            input_bytes,
            output_bytes,
        )
    }

    pub fn agent_to_client<T>(
        agent: T,
        input_bytes: impl Unpin + AsyncWrite,
        output_bytes: impl Unpin + AsyncRead,
    ) -> (
        Self,
        impl Future<Output = ()>,
        impl Future<Output = Result<()>>,
    )
    where
        T: 'static + Send + Sync + Agent,
    {
        let agent = Arc::new(agent);
        Self::new(
            Box::new(move |method, params| {
                let agent = agent.clone();
                async move { agent.call(method, params).await }.boxed()
            }),
            input_bytes,
            output_bytes,
        )
    }

    fn new(
        request_handler: RequestHandler,
        input_bytes: impl Unpin + AsyncWrite,
        output_bytes: impl Unpin + AsyncRead,
    ) -> (
        Self,
        impl Future<Output = ()>,
        impl Future<Output = Result<()>>,
    ) {
        let state = Arc::new(Mutex::new(ConnectionState {
            response_senders: Default::default(),
        }));
        let (input_tx, input_rx) = futures::channel::mpsc::unbounded();
        let (output_tx, output_rx) = futures::channel::mpsc::unbounded();
        let this = Self {
            state: state.clone(),
            input_tx: input_tx.clone(),
            next_id: AtomicI32::new(0),
        };
        let handle_incoming = handle_incoming(request_handler, state, input_tx, output_rx);
        let handle_io = handle_io(input_bytes, output_bytes, input_rx, output_tx);
        (this, handle_incoming, handle_io)
    }

    pub fn request<R: Request>(&self, params: R) -> impl Future<Output = Result<R::Response>> {
        let id = self.next_id.fetch_add(1, SeqCst);
        let (tx, rx) = oneshot::channel();
        self.state.lock().response_senders.insert(id, tx);
        self.input_tx
            .unbounded_send(
                serde_json::to_string(&AnyMessage::Request {
                    id,
                    method: R::METHOD.into(),
                    params: serde_json::to_string(&params).unwrap().into(),
                })
                .unwrap()
                .into(),
            )
            .ok();
        async move {
            let result = rx.await??;
            Ok(serde_json::from_str(&result)?)
        }
    }
}

async fn handle_io(
    mut input_bytes: impl Unpin + AsyncWrite,
    output_bytes: impl Unpin + AsyncRead,
    mut input_rx: UnboundedReceiver<Box<str>>,
    output_tx: UnboundedSender<Box<str>>,
) -> Result<()> {
    let mut output_reader = BufReader::new(output_bytes);
    let mut chunk = String::new();
    loop {
        select_biased! {
            line = input_rx.next() => {
                if let Some(line) = line {
                    input_bytes.write_all(line.as_bytes()).await.ok();
                    input_bytes.write(b"\n").await.ok();
                } else {
                    break;
                }
            }
            bytes_read = output_reader.read_line(&mut chunk).fuse() => {
                if bytes_read? == 0 {
                    break
                }
                if output_tx.unbounded_send(chunk.into()).is_err() {
                    break
                }
                chunk = String::new();
            }
        }
    }
    Ok(())
}

async fn handle_incoming(
    incoming_handler: RequestHandler,
    state: Arc<Mutex<ConnectionState>>,
    input_tx: UnboundedSender<Box<str>>,
    mut output_rx: UnboundedReceiver<Box<str>>,
) {
    while let Some(message) = output_rx.next().await {
        // todo! move json parsing to background io loop
        match serde_json::from_str(&message) {
            Ok(msg) => match msg {
                AnyMessage::Request { id, method, params } => {
                    let result = incoming_handler(method, params).await;
                    match result {
                        Ok(result) => {
                            input_tx
                                .unbounded_send(
                                    serde_json::to_string(&AnyMessage::OkResponse { id, result })
                                        .unwrap()
                                        .into(),
                                )
                                .ok();
                        }
                        Err(error) => {
                            input_tx
                                .unbounded_send(
                                    serde_json::to_string(&AnyMessage::ErrorResponse {
                                        id,
                                        error: error.to_string(),
                                    })
                                    .unwrap()
                                    .into(),
                                )
                                .ok();
                        }
                    }
                }
                AnyMessage::OkResponse { id, result } => {
                    let state = &mut state.lock();
                    if let Some(sender) = state.response_senders.remove(&id) {
                        sender.send(Ok(result)).ok();
                    }
                }
                AnyMessage::ErrorResponse { id, error } => {
                    let state = &mut state.lock();
                    if let Some(sender) = state.response_senders.remove(&id) {
                        sender.send(Err(anyhow!("{}", error))).ok();
                    }
                }
            },
            Err(err) => {
                eprintln!("error: {err:?} - {}", message);
                break;
            }
        }
    }
}
