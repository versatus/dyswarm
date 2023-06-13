use std::{
    fmt::Debug,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    internal::{engine::Engine, EngineConfig},
    server::handler::Handler,
    types::{DyswarmError, Message},
    Result,
};

fn null_bytes() -> (Bytes, Bytes, Bytes) {
    (Bytes::new(), Bytes::new(), Bytes::new())
}

#[derive(Debug)]
pub struct ServerConfig {
    pub addr: SocketAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        Self { addr }
    }
}

#[derive(Debug)]
pub struct Server {
    config: ServerConfig,
    engine: Engine,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let engine_config = EngineConfig {
            addr: config.addr,
            ..Default::default()
        };

        let engine = Engine::new(engine_config).await?;

        Ok(Self { config, engine })
    }

    /// Returns a new Dyswarm server with a custom p2p engine
    pub fn new_with_engine(engine: Engine) -> Self {
        let config = ServerConfig::default();
        Self { config, engine }
    }

    /// Returns the public address the internal engine is listening to incomming connetctions on.
    pub fn public_addr(&self) -> SocketAddr {
        self.engine.public_addr()
    }

    /// Starts listening to network events and calls the provided handler.
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
                        return;
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
            handle,
            cancel_token,
        };

        Ok(server_handle)
    }
}

#[derive(Debug)]
pub struct ServerHandle {
    handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

impl ServerHandle {
    pub async fn stop(self) {
        self.cancel_token.cancel();
        if let Err(err) = self.handle.await {
            tracing::error!("Error stopping server: {:?}", err);
        }
    }
}
