use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Debug,
    marker::PhantomData,
    net::SocketAddr,
};

use crate::{
    engine::engine::{Engine, EngineConfig},
    types::{BroadcastError, DyswarmError, Message, Result},
};
use bytes::Bytes;
use qp2p::Connection;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::UdpSocket, task::JoinSet};
use tracing::{error, info};

#[derive(Debug)]
pub struct Config {
    pub addr: SocketAddr,
    pub known_peers: Vec<SocketAddr>,
}

#[derive(Debug)]
pub struct Client {
    config: Config,
    engine: Engine,
}

#[derive(Debug, Clone, Default)]
pub struct BroadcastConfig {
    pub unreliable: bool,
}

#[derive(Debug, Clone)]
pub struct BroadcastArgs<D>
where
    D: Default + Debug + Clone,
{
    pub config: BroadcastConfig,
    pub message: Message<D>,
    pub peer_list: BTreeMap<SocketAddr, Connection>,
    // TODO: merge both lists into one entity
    pub raptor_list: HashSet<SocketAddr>,
    pub erasure_count: u32,
}

impl Client {
    pub async fn new(config: Config) -> Result<Self> {
        let engine_config = EngineConfig::default();
        let engine = Engine::new(engine_config).await?;

        Ok(Client { config, engine })
    }

    pub async fn add_peers(&mut self, peers: Vec<SocketAddr>) -> Result<()> {
        self.engine.add_peer_connections(peers).await
    }

    pub async fn remove_peers(&mut self, peers: Vec<SocketAddr>) -> Result<()> {
        self.engine.remove_peer_connections(peers)
    }

    /// This function takes a message and sends it to all the peers in the
    /// peer list
    ///
    /// Arguments:
    ///
    /// * `message`: Message - The message to be broadcasted
    ///
    #[tracing::instrument]
    pub async fn broadcast<D>(&self, args: BroadcastArgs<D>) -> Result<()>
    where
        D: Default + Debug + Clone + Serialize + DeserializeOwned,
    {
        // Either call quic_broadcast or unreliable_broadcast
        if args.config.unreliable {
            let addr = "127.0.0.1:0".parse::<SocketAddr>().map_err(|err| {
                DyswarmError::Other("Unable to parse address for unreliable broadcast".to_string())
            })?;

            self.unreliable_broadcast(args.message, args.erasure_count, addr)
                .await
        } else {
            self.quic_broadcast(args.message).await
        }
    }

    /// This function takes a message and sends it to all the peers in the
    /// peer list
    ///
    /// Arguments:
    ///
    /// * `message`: Message - The message to be broadcasted
    /// * `peer_connection_list`: BTreeMap<SocketAddr, Connection> - The list of peers to which the message is to be sent
    ///
    async fn quic_broadcast<D>(&self, message: Message<D>) -> Result<()>
    where
        D: Default + Debug + Clone + Serialize + DeserializeOwned,
    {
        let message_bytes = Bytes::from(message.as_bytes());

        self.engine.quic_broadcast(message_bytes).await
    }

    /// The function takes a message and an erasure count as input and splits
    /// the message into packets
    /// and sends them to the peers
    ///
    /// Arguments:
    ///
    /// * `message`: The message to be broadcasted.
    /// * `erasure_count`: The number of packets that can be lost and still be
    ///   able to reconstruct the
    /// original message.
    ///
    #[tracing::instrument]
    pub async fn unreliable_broadcast<D>(
        &self,
        message: Message<D>,
        erasure_count: u32,
        addr: SocketAddr,
    ) -> Result<()>
    where
        D: Default + Debug + Clone + Serialize + DeserializeOwned,
    {
        let message_bytes = Bytes::from(message.as_bytes());

        let udp_socket = UdpSocket::bind(addr)
            .await
            .map_err(|err| DyswarmError::Other(err.to_string()))?;

        self.engine
            .unreliable_broadcast(message_bytes, erasure_count, udp_socket)
            .await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn send_data_via_quic<D>(&self, message: Message<D>, addr: SocketAddr) -> Result<()>
    where
        D: Default + Debug + Clone + Serialize + DeserializeOwned,
    {
        let message_bytes = Bytes::from(message.as_bytes());
        self.engine.send_data_via_quic(message_bytes, addr).await
    }
}
