use std::net::SocketAddr;

use qp2p::{ConnectionError, EndpointError, SendError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use udp2p::node::peer_id::PeerId;

/// `Topology` is a struct that contains the number of master nodes, the number
/// of quorum nodes, and the miner network id.
///
/// Properties:
///
/// * `num_of_master_nodes`: The number of master nodes in the network.
/// * `num_of_quorum_nodes`: The number of nodes in the quorum.
/// * `miner_network_id`: The peer id of the miner node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct Topology {
    pub num_of_master_nodes: usize,
    pub num_of_quorum_nodes: usize,
    pub miner_network_id: PeerId,
}

impl Topology {
    pub fn new(
        num_of_master_nodes: usize,
        num_of_quorum_nodes: usize,
        miner_network_id: PeerId,
    ) -> Self {
        Self {
            num_of_master_nodes,
            num_of_quorum_nodes,
            miner_network_id,
        }
    }
}

#[derive(Debug)]
pub enum BroadcastStatus {
    ConnectionEstablished,
    Success,
}
