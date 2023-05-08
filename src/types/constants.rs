use std::{fmt, net::SocketAddr};

use serde::{Deserialize, Serialize};

/// The unit of time within VRRB.
/// It lasts for some number
pub type Epoch = u128;

pub const GENESIS_EPOCH: Epoch = 0;
pub const GROSS_UTILITY_PERCENTAGE: f64 = 0.01;
pub const PERCENTAGE_CHANGE_SUPPLY_CAP: f64 = 0.25;

// Time-related helper constants
pub const NANO: u128 = 1;
pub const MICRO: u128 = NANO * 1000;
pub const MILLI: u128 = MICRO * 1000;
pub const SECOND: u128 = MILLI * 1000;
pub const VALIDATOR_THRESHOLD: f64 = 0.60;
pub const NUMBER_OF_NETWORK_PACKETS: usize = 32;
pub const DEFAULT_CONNECTION_TIMEOUT_IN_SECS: u64 = 2;
pub const RAPTOR_DECODER_CACHE_LIMIT: usize = 10000;
pub const RAPTOR_DECODER_CACHE_TTL_IN_SECS: u64 = 1800000;
pub const CONNECTION_CLOSED: &str = "The connection was closed intentionally by qp2p.";

pub type ByteVec = Vec<u8>;
pub type ByteSlice<'a> = &'a [u8];

pub type PeerId = ByteVec;
pub type NodeIdx = u16;
pub type MessageId = uuid::Uuid;
pub const MAX_TRANSMIT_SIZE: usize = 1024;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerData {
    pub address: SocketAddr,
    pub peer_id: PeerId,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SyncPeerData {
    pub address: SocketAddr,
    pub raptor_udp_port: u16,
    pub quic_port: u16,
}

/// Maximum over-the-wire size of a Transaction
///   1280 is IPv6 minimum MTU
///   40 bytes is the size of the IPv6 header
///   8 bytes is the size of the fragment header
pub(crate) const MTU_SIZE: usize = 1280;

pub(crate) const BATCH_ID_SIZE: usize = 32;

pub(crate) const PACKET_SNO: usize = 4;

pub(crate) const FLAGS: usize = 1;

///Index at which actual payload starts.
pub(crate) const DECODER_DATA_INDEX: usize = 40;

///How many packets to recieve from socket in single system call
pub(crate) const NUM_RCVMMSGS: usize = 32;

///   40 bytes is the size of the IPv6 header
///   8 bytes is the size of the fragment header
///   True payload size ,or the size of single packet that will be written to or
/// read from socket
pub(crate) const PAYLOAD_SIZE: usize = MTU_SIZE - PACKET_SNO - BATCH_ID_SIZE - FLAGS - 40 - 8;
