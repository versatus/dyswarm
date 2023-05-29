use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
    time::Duration,
};

use crate::{
    internal::{cache::Cache, forwarder::packet_forwarder},
    types::{CONNECTION_CLOSED, DEFAULT_CONNECTION_TIMEOUT_IN_SECS, MTU_SIZE},
};
use crate::{
    internal::{reassembler::reassemble_packets, receiver::recv_mmsg},
    types::{
        DyswarmError, BATCH_ID_SIZE, NUM_RCVMMSGS, RAPTOR_DECODER_CACHE_LIMIT,
        RAPTOR_DECODER_CACHE_TTL_IN_SECS,
    },
};
use crate::{types::NUMBER_OF_NETWORK_PACKETS, Result};

use crate::internal::{generate_batch_id, split_into_packets};
use bytes::Bytes;
use crossbeam_channel::{unbounded, Sender};
pub use qp2p::{
    Config, Connection, ConnectionIncoming, Endpoint,
    IncomingConnections as IncomingConnectionsReceiver, RetryConfig,
};

use raptorq::Decoder;
use tokio::{net::UdpSocket, task::JoinSet};
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct Engine {
    peer_connection_list: HashMap<SocketAddr, Connection>,
    raptor_list: HashSet<SocketAddr>,
    endpoint: Endpoint,
    conn_rx: IncomingConnectionsReceiver,
    raptor_udp_port: u16,
    raptor_num_packet_blast: usize,
}

pub type ConnectionApiPair = (Connection, ConnectionIncoming);

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub addr: SocketAddr,
    pub known_peers: Vec<SocketAddr>,
    pub peer_connection_list: HashMap<SocketAddr, Connection>,
    pub raptor_list: HashSet<SocketAddr>,
    pub raptor_num_packet_blast: usize,
    pub raptor_udp_port: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        Self {
            addr,
            // TODO: merge known_peers, peer_connection_list and raptor_list into a single
            // entity
            known_peers: Default::default(),
            peer_connection_list: Default::default(),
            raptor_list: Default::default(),

            raptor_num_packet_blast: NUMBER_OF_NETWORK_PACKETS,
            raptor_udp_port: Default::default(),
        }
    }
}

impl EngineConfig {
    //
}

impl Engine {
    pub async fn new(config: EngineConfig) -> Result<Self> {
        let (endpoint, conn_rx, _) = Self::new_endpoint(config.addr, &config.known_peers).await?;

        // TODO: fix these
        Ok(Self {
            peer_connection_list: config.peer_connection_list,
            raptor_list: config.raptor_list,
            endpoint,
            conn_rx,
            raptor_udp_port: config.raptor_udp_port,
            raptor_num_packet_blast: config.raptor_num_packet_blast,
        })
    }

    async fn new_endpoint(
        addr: SocketAddr,
        known_peers: &[SocketAddr],
    ) -> Result<(
        Endpoint,
        IncomingConnectionsReceiver,
        Option<ConnectionApiPair>,
    )> {
        let peer_config = Config {
            retry_config: RetryConfig {
                retrying_max_elapsed_time: Duration::from_millis(500),
                ..RetryConfig::default()
            },
            keep_alive_interval: Some(Duration::from_secs(5)),
            ..Config::default()
        };

        let (endpoint, incoming_connections_receiver, conn_opts) =
            Endpoint::new_peer(addr, known_peers, peer_config).await?;

        Ok((endpoint, incoming_connections_receiver, conn_opts))
    }

    pub fn public_addr(&self) -> SocketAddr {
        self.endpoint.public_addr()
    }

    /// This function removes a peer connection from the peer connection list
    ///
    /// Arguments:
    ///
    /// * `address`: The address of the peer to be removed.
    #[tracing::instrument]
    pub fn remove_peer_connections(&mut self, addresses: Vec<SocketAddr>) -> Result<()> {
        for addr in addresses.iter() {
            self.peer_connection_list.retain(|address, connection| {
                info!("closed connection with: {addr}");
                connection.close(Some(String::from(CONNECTION_CLOSED)));
                address != addr
            });
        }

        Ok(())
    }

    /// This function takes a vector of socket addresses and attempts to
    /// connect to each one. If the
    /// connection is successful, it adds the connection to the peer connection
    /// list
    ///
    /// Arguments:
    ///
    /// * `address`: A vector of SocketAddr, which is the address of the peer
    ///   you want to connect to.
    #[tracing::instrument]
    pub async fn add_peer_connections(&mut self, addresses: Vec<SocketAddr>) -> Result<()> {
        for addr in addresses.iter() {
            let connection_result = self.endpoint.connect_to(addr).timeout().await;

            match connection_result {
                Ok(con_result) => {
                    let (connection, _) = con_result.map_err(|err| {
                        error!("Failed to connect with {addr}: {err}");

                        DyswarmError::Connection(err)
                    })?;

                    self.peer_connection_list
                        .insert(addr.to_owned(), connection);
                }
                Err(e) => {
                    error!("Connection error {addr}: {e}");
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    pub fn add_raptor_peers(&mut self, address: Vec<SocketAddr>) {
        self.raptor_list.extend(address)
    }

    #[tracing::instrument]
    pub async fn send_data_via_quic(&self, message_bytes: Bytes, addr: SocketAddr) -> Result<()> {
        let endpoint = self.endpoint.clone();

        let (conn, _) = endpoint.connect_to(&addr).await?;

        let msg = (Bytes::new(), Bytes::new(), message_bytes.clone());

        conn.send(msg).await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn quic_broadcast(&self, message_bytes: Bytes) -> crate::types::Result<()> {
        let mut set = JoinSet::new();

        if self.peer_connection_list.is_empty() {
            return Err(DyswarmError::NoPeers);
        }

        let byte_len = message_bytes.len();

        for (addr, conn) in self.peer_connection_list.clone().into_iter() {
            let message_bytes = message_bytes.clone();

            set.spawn(async move {
                let msg = (Bytes::new(), Bytes::new(), message_bytes.clone());

                match conn.send(msg).await {
                    Ok(_) => {
                        info!("sent {byte_len} bytes to {addr}");
                    }
                    Err(err) => {
                        error!("error: {err}");
                    }
                }
            });
        }

        while let Some(fut) = set.join_next().await {
            match fut {
                Ok(_) => {
                    debug!("sent {byte_len} bytes");
                }
                Err(err) => {
                    error!("error: {err}");
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn unreliable_broadcast(
        &self,
        message_bytes: Bytes,
        erasure_count: u32,
        udp_socket: UdpSocket,
    ) -> Result<()> {
        info!("broadcasting from {:?}", &udp_socket);

        let byte_len = message_bytes.len();
        let batch_id = generate_batch_id();
        let chunks = split_into_packets(&message_bytes, batch_id, erasure_count);

        let mut set = JoinSet::new();

        let udp_socket = Arc::new(udp_socket);

        if self.raptor_list.is_empty() {
            return Err(DyswarmError::NoPeers);
        }

        let socket = udp_socket.clone();

        for (packet_index, packet) in chunks.iter().enumerate() {
            let socket = socket.clone();

            let addresses =
                self.get_address_for_packet_shards(packet_index, self.raptor_list.clone());

            for address in addresses.into_iter() {
                let packet = packet.clone();
                let socket = socket.clone();

                set.spawn(async move {
                    let addr = address.to_string();

                    socket.send_to(&packet, addr.clone()).await;
                });

                if set.len() >= self.raptor_num_packet_blast {
                    match set.join_next().await {
                        Some(fut) => {
                            if fut.is_err() {
                                error!("Sending future is not ready yet")
                            }
                        }
                        None => error!("Sending future is not ready yet"),
                    }
                }
            }
        }

        while let Some(fut) = set.join_next().await {
            match fut {
                Ok(_) => {
                    debug!("sent {byte_len} bytes");
                }
                Err(err) => {
                    error!("error: {err}");
                }
            }
        }

        Ok(())
    }

    /// It receives packets from the socket, and sends them to the reassembler
    /// thread
    ///
    /// Arguments:
    ///
    /// * `port`: The port on which the node is listening for incoming packets.
    ///
    /// Returns:
    ///
    /// a future that resolves to a result. The result is either an error or a
    /// unit.
    #[tracing::instrument]
    pub async fn process_received_packets(
        &self,
        port: u16,
        batch_sender: Sender<Bytes>,
        udp_socket_receiver: UdpSocket,
        raptor_list: HashSet<SocketAddr>,
    ) -> Result<()> {
        info!("listening on {:?}", udp_socket_receiver);

        let buf = [0; MTU_SIZE];
        let (reassembler_channel_tx, reassembler_channel_rx) = unbounded();
        let (forwarder_tx, forwarder_rx) = unbounded();
        let mut batch_id_store: HashSet<[u8; BATCH_ID_SIZE]> = HashSet::new();

        let mut decoder_hash_cache: Cache<[u8; BATCH_ID_SIZE], (usize, Decoder)> =
            Cache::new(RAPTOR_DECODER_CACHE_LIMIT, RAPTOR_DECODER_CACHE_TTL_IN_SECS);

        thread::spawn({
            let assemble_tx = reassembler_channel_tx.clone();
            let fwd_tx = forwarder_tx.clone();
            let batch_tx = batch_sender.clone();

            move || {
                reassemble_packets(
                    reassembler_channel_rx,
                    &mut batch_id_store,
                    &mut decoder_hash_cache,
                    fwd_tx.clone(),
                    batch_tx.clone(),
                );

                // TODO: refactor these drops
                // drop(assemble_send);
                // drop(fwd_send);
                // drop(batch_send);
            }
        });

        let mut nodes_ips_except_self = vec![];
        if raptor_list.is_empty() {
            return Err(DyswarmError::NoPeers);
        }

        raptor_list
            .iter()
            .for_each(|addr| nodes_ips_except_self.push(addr.to_string().as_bytes().to_vec()));

        let port = self.raptor_udp_port;

        let forwarder_udp_socket = UdpSocket::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))
        .await
        .map_err(|err| {
            error!("UDP port {port} already in use");
            DyswarmError::Other(err.to_string())
        })?;

        let receiver_udp_socket = UdpSocket::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))
        .await
        .map_err(|err| {
            error!("UDP port {port} already in use");
            DyswarmError::Other(err.to_string())
        })?;

        thread::spawn(move || {
            packet_forwarder(
                forwarder_rx,
                nodes_ips_except_self,
                port,
                forwarder_udp_socket,
            )
        });

        // TODO: implement stop condition
        loop {
            let mut receive_buffers = [buf; NUM_RCVMMSGS];
            // Receiving a batch of packets from the socket.
            if let Ok(res) = recv_mmsg(&receiver_udp_socket, receive_buffers.borrow_mut()).await {
                if !res.is_empty() {
                    let mut i = 0;
                    for buf in &receive_buffers {
                        if let Some(packets_info) = res.get(i) {
                            let _ = reassembler_channel_tx.send((*buf, packets_info.1));
                            i += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    pub fn get_incoming_connections(&mut self) -> &mut IncomingConnectionsReceiver {
        &mut self.conn_rx
    }

    fn get_address_for_packet_shards(
        &self,
        packet_index: usize,
        raptor_list: HashSet<SocketAddr>,
    ) -> Vec<SocketAddr> {
        let total_peers = raptor_list.len();
        let mut addresses = Vec::new();
        let number_of_peers = (total_peers as f32 * 0.10).ceil() as usize;
        let raptor_list_cloned: Vec<&SocketAddr> = raptor_list.iter().collect();

        for i in 0..number_of_peers {
            if let Some(address) = raptor_list_cloned.get(packet_index % (total_peers + i)) {
                // TODO: refactor this double owning
                addresses.push(address.to_owned().to_owned());
            }
        }

        addresses
    }
}

pub trait Timeout: Sized {
    fn timeout(self) -> tokio::time::Timeout<Self>;
}

impl<F: std::future::Future> Timeout for F {
    fn timeout(self) -> tokio::time::Timeout<Self> {
        tokio::time::timeout(
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_IN_SECS),
            self,
        )
    }
}
