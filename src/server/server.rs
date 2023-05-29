use std::fmt::Debug;

use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    internal::engine::Engine,
    server::handler::Handler,
    types::{DyswarmError, Message},
    Result,
};

fn null_bytes() -> (Bytes, Bytes, Bytes) {
    (Bytes::new(), Bytes::new(), Bytes::new())
}

#[derive(Debug)]
pub struct Server {
    engine: Engine,
}

impl Server {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }

    pub async fn run<H, D>(mut self, handler: H) -> Result<ServerHandle>
    where
        D: Debug + Default + Serialize + DeserializeOwned + Clone + Send,
        H: Handler<D> + std::marker::Send + std::marker::Sync + 'static,
    {
        let cancel_token = CancellationToken::new();
        let cloned_token = cancel_token.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cloned_token.cancelled() => {
                        tracing::info!("Server received stop signal");
                        tracing::info!("Shutting down...");
                        //
                        // TODO: call handler.cleanup() here
                        //
                        tracing::info!("Server shutdown complete");
                    }
                    Some((_, mut conn_rx)) = self.engine.get_incoming_connections().next() => {
                        let res = conn_rx
                            .next()
                            .await
                            .map_err(|err| {
                                DyswarmError::Other(format!(
                                    "unable to listen for new connections: {err}"
                                ))
                            })
                            .unwrap_or(Some(null_bytes()))
                            .unwrap_or(null_bytes());

                        let (_, _, raw_bytes) = res;

                        let message: Message<D> = Message::from(raw_bytes);

                        if let Err(err) = handler.handle(message).await {
                            tracing::error!("Error handling connection: {:?}", err);
                        }
                    }
                }
            }
        });

        let server_handle = ServerHandle {
            _handle: handle,
            cancel_token,
        };

        Ok(server_handle)
    }
}

#[derive(Debug)]
pub struct ServerHandle {
    _handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

impl ServerHandle {
    pub fn stop(self) {
        self.cancel_token.cancel();
    }
}
