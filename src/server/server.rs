use std::fmt::Debug;

use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tokio::task::JoinHandle;

use crate::{
    engine::engine::Engine,
    server::handler::Handler,
    types::{DyswarmError, Message},
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

    pub async fn run<H, D>(mut self, handler: H) -> crate::Result<JoinHandle<()>>
    where
        D: Debug + Default + Serialize + DeserializeOwned + Clone + Send,
        H: Handler<D> + std::marker::Send + std::marker::Sync + 'static,
    {
        let handle = tokio::spawn(async move {
            loop {
                if let Some((_, mut conn_rx)) = self.engine.get_incoming_connections().next().await
                {
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
        });

        Ok(handle)
    }
}
