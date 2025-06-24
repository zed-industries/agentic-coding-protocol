#[cfg(test)]
mod acp_tests;
mod schema;

use anyhow::{Result, anyhow};
use futures::{
    FutureExt as _, StreamExt as _,
    channel::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::BoxFuture,
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
        input_tx: UnboundedSender<Box<str>>,
        output_rx: UnboundedReceiver<Box<str>>,
    ) -> (Self, impl Future<Output = ()>)
    where
        C: 'static + Send + Sync + Client,
    {
        let client = Arc::new(client);
        Self::new(
            Box::new(move |method, params| {
                let client = client.clone();
                async move { client.call(method, params).await }.boxed()
            }),
            input_tx,
            output_rx,
        )
    }

    pub fn agent_to_client<T>(
        agent: T,
        input_tx: UnboundedSender<Box<str>>,
        output_rx: UnboundedReceiver<Box<str>>,
    ) -> (Self, impl Future<Output = ()>)
    where
        T: 'static + Send + Sync + Agent,
    {
        let agent = Arc::new(agent);
        Self::new(
            Box::new(move |method, params| {
                let agent = agent.clone();
                async move { agent.call(method, params).await }.boxed()
            }),
            input_tx,
            output_rx,
        )
    }

    fn new(
        request_handler: RequestHandler,
        input_tx: UnboundedSender<Box<str>>,
        output_rx: UnboundedReceiver<Box<str>>,
    ) -> (Self, impl Future<Output = ()>) {
        let state = Arc::new(Mutex::new(ConnectionState {
            response_senders: Default::default(),
        }));
        let this = Self {
            state: state.clone(),
            input_tx: input_tx.clone(),
            next_id: AtomicI32::new(0),
        };
        let handle_io = handle_incoming(request_handler, state, input_tx, output_rx);
        (this, handle_io)
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

async fn handle_incoming(
    incoming_handler: RequestHandler,
    state: Arc<Mutex<ConnectionState>>,
    input_tx: UnboundedSender<Box<str>>,
    mut output_rx: UnboundedReceiver<Box<str>>,
) {
    while let Some(message) = output_rx.next().await {
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
            }
        }
    }
}
