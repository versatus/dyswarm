use std::net::SocketAddr;
use std::sync::Arc;
use std::{collections::HashSet, io::Read, str};

// use block::Block;
use crate::engine::cache::Cache;
use crate::types::constants::BATCH_ID_SIZE;
use crate::types::Result;
use crate::types::PAYLOAD_SIZE;
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use rand::{distributions::Alphanumeric, thread_rng, Rng, RngCore};
use raptorq::{Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

use crate::types::constants::NUM_RCVMMSGS;

// /// Below is the type that shall be used to broadcast RaptorQ Data
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub enum RaptorBroadCastedData {
//     Block(Vec<u8>),
//     // Block(Block),
// }
//
/// It takes a batch id, a sequence number, and a payload, and returns a packet
///
/// Arguments:
///
/// * `batch_id`: This is the batch id that we're sending.
/// * `payload`: the data to be sent
///
/// Returns:
///
/// A vector of bytes
pub fn create_packet(symbol_size: u16, batch_id: [u8; BATCH_ID_SIZE], payload: Vec<u8>) -> Vec<u8> {
    let mut mtu: Vec<u8> = vec![];

    // empty byte for raptor coding length
    // doing the plus one since raptor is returning minus 1 length.

    mtu.push(0_u8);
    // forward-flag at the beginning
    mtu.push(1_u8);

    //Size of Payload

    mtu.extend((payload.len() as u32).to_le_bytes());

    mtu.extend(symbol_size.to_le_bytes());

    for id in batch_id {
        mtu.push(id);
    }
    mtu.extend_from_slice(&payload);
    mtu
}
//
/// `split_into_packets` takes a `full_list` of bytes, a `batch_id` and an
/// `erasure_count` and returns a `Vec<Vec<u8>>` of packets
///
/// Arguments:
///
/// * `full_list`: The list of bytes to be split into packets
/// * `batch_id`: This is a unique identifier for the batch of packets.
/// * `erasure_count`: The number of packets that can be lost and still be able
///   to recover the original data.
pub fn split_into_packets(
    full_list: &[u8],
    batch_id: [u8; BATCH_ID_SIZE],
    erasure_count: u32,
) -> Vec<Vec<u8>> {
    let packet_holder = encode_into_packets(full_list, erasure_count);

    let mut headered_packets: Vec<Vec<u8>> = vec![];
    for (_, ep) in packet_holder.1.into_iter().enumerate() {
        headered_packets.push(create_packet(packet_holder.0, batch_id, ep))
    }

    debug!("Packets len {:?}", headered_packets.len());
    headered_packets
}

/// It takes a list of bytes and an erasure count, and returns a list of packets
///
/// Arguments:
///
/// * `unencoded_packet_list`: This is the list of packets that we want to
///   encode.
/// * `erasure_count`: The number of packets that can be lost and still be able
///   to recover the original
/// data.
///
/// Returns:
///
/// A vector of vectors of bytes.
pub fn encode_into_packets(
    unencoded_packet_list: &[u8],
    erasure_count: u32,
) -> (u16, Vec<Vec<u8>>) {
    let encoder = Encoder::with_defaults(unencoded_packet_list, (PAYLOAD_SIZE) as u16);
    println!("encoder :{:?}", encoder.get_config().symbol_size());

    let packets: Vec<Vec<u8>> = encoder
        .get_encoded_packets(erasure_count)
        .iter()
        .map(|packet| packet.serialize())
        .collect();
    (encoder.get_config().symbol_size(), packets)
}

/// It takes a packet and returns the batch id
///
/// Arguments:
///
/// * `packet`: The packet that we want to extract the batch id from.
///
/// Returns:
///
/// The batch_id is being returned.
// TODO: Make sure this is correct
// Seems like batch_id is of length = BATCH_ID_SIZE but is only overwritten at
// batch_id[0..BATCH_ID_SIZE - 3]. Last 3 elements will always be 0 here
#[allow(clippy::manual_memcpy)]
pub fn get_batch_id(packet: &[u8; 1280]) -> [u8; BATCH_ID_SIZE] {
    let mut batch_id: [u8; BATCH_ID_SIZE] = [0; BATCH_ID_SIZE];
    let mut chunk_no: usize = 0;
    for i in 10..(BATCH_ID_SIZE + 10) {
        batch_id[chunk_no] = packet[i];
        chunk_no += 1;
    }
    // The above equals to
    // batch_id[..(BATCH_ID_SIZE - 3)].copy_from_slice(&packet[6..(BATCH_ID_SIZE +
    // 3)]);
    batch_id
}

/// It takes a packet as an argument, and returns the length of the payload as
/// an integer
///
/// Arguments:
///
/// * `packet`: The packet that we want to get the payload length from.
///
/// Returns:
///
/// The length of the payload in bytes.
pub fn get_payload_length(packet: &[u8; 1280]) -> u32 {
    let mut payload_len_bytes: [u8; 4] = [0; 4];

    payload_len_bytes[..4].copy_from_slice(&packet[2..6]);
    u32::from_le_bytes(payload_len_bytes)
}

pub fn get_symbol_size(packet: &[u8; 1280]) -> u16 {
    let mut symbol_size_bytes: [u8; 2] = [0; 2];
    symbol_size_bytes[..2].copy_from_slice(&packet[6..8]);
    u16::from_le_bytes(symbol_size_bytes)
}

/// Generate a random 32 byte batch id
pub fn generate_batch_id() -> [u8; BATCH_ID_SIZE] {
    let mut x = [0_u8; BATCH_ID_SIZE];
    thread_rng().fill_bytes(&mut x);
    let s: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(BATCH_ID_SIZE)
        .map(char::from)
        .collect();

    x.copy_from_slice(s.as_bytes());
    x
}
//
// /// It reads the contents of a file into a byte array
// ///
// /// Arguments:
// ///
// /// * `file_path`: The path to the file you want to read.
// pub fn read_file(file_path: PathBuf) -> Result<Vec<u8>> {
//     let mut buffer;
//     match fs::metadata(&file_path) {
//         Ok(metadata) => {
//             buffer = vec![0; metadata.len() as usize];
//             if let Ok(mut file) = File::open(file_path) {
//                 match file.read_exact(&mut buffer) {
//                     Ok(()) => {}
//                     Err(e) => {
//                         error!("Error occured while reading a file, details {}", e);
//                         return Err(e);
//                     }
//                 }
//             }
//         }
//         Err(e) => {
//             error!(
//                 "Error occured while reading metadata of file, details {}",
//                 e
//             );
//             return Err(e);
//         }
//     }
//     Ok(buffer)
// }
//
// /// It receives a tuple of a string and a vector of bytes from a channel, and
// /// writes the vector of bytes to a file whose name is the string
// ///
// /// Arguments:
// ///
// /// * `batch_recv`: Receiver<(String, Vec<u8>)>
// pub fn batch_writer(batch_recv: Receiver<(String, Vec<u8>)>) {
//     loop {
//         match batch_recv.recv() {
//             Ok((batch_id, contents)) => {
//                 let batch_fname = format!("{}.BATCH", batch_id);
//                 match fs::write(batch_fname, contents) {
//                     Ok(_) => {}
//                     Err(e) => {
//                         error!("Error occured while write data to file, details {}", e)
//                     }
//                 }
//             }
//             Err(_e) => {
//                 continue;
//             }
//         };
//     }
// }
//

// /// A Basic error unit struct to return in the event a series of packets cannot
// /// be reassembled into a type
// #[derive(Debug)]
// pub struct NotCompleteError;
//
// /// The basic structure that is converted into bytes to be sent across the
// /// network
// //TODO: Replace standard types with custom types to make it more obvious what their
// // purposes are.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Packet {
//     pub id: Vec<u8>,
//     pub source: Option<Vec<u8>>,
//     pub data: Vec<u8>,
//     pub size: Vec<u8>,
//     pub packet_number: Vec<u8>,
//     pub total_packets: Vec<u8>,
//     pub return_receipt: u8,
// }
//
// impl Packet {
//     /// Assembles and returns a new packet
//     //TODO: Convert Vec<u8> and other standard types to custom types that are more
//     // descriptive of their purpose
//     pub fn new(
//         id: Vec<u8>,
//         source: Option<Vec<u8>>,
//         data: Vec<u8>,
//         size: Vec<u8>,
//         packet_number: Vec<u8>,
//         total_packets: Vec<u8>,
//         return_receipt: u8,
//     ) -> Packet {
//         Packet {
//             id,
//             source,
//             data,
//             size,
//             packet_number,
//             total_packets,
//             return_receipt,
//         }
//     }
//
//     /// Converts a packet number into an array of bytes (8 bytes)
//     pub fn convert_packet_number(self) -> [u8; 8] {
//         self.packet_number
//             .try_into()
//             .unwrap_or_else(|_| panic!("Expected a Vec of length 8"))
//     }
//
//     /// Converts the total number of packets into an array of bytes (8 bytes)
//     pub fn convert_total_packets(self) -> [u8; 8] {
//         self.total_packets
//             .try_into()
//             .unwrap_or_else(|_| panic!("Expected a Vec of length 8"))
//     }
//
//     /// Returns true if the total number of packets is only 1
//     pub fn is_complete(&self) -> bool {
//         usize::from_be_bytes(self.clone().convert_total_packets()) == 1
//     }
//
//     /// Returns a vector of bytes from a Packet
//     pub fn as_bytes(&self) -> Vec<u8> {
//         self.to_string().as_bytes().to_vec()
//     }
//
//     /// Serializes a Packet into a string
//     // TODO: Is this fine?
//     #[allow(clippy::inherent_to_string)]
//     pub fn to_string(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
//
//     /// Deserializes an array of bytes into a Packet
//     pub fn from_bytes(data: &[u8]) -> Packet {
//         serde_json::from_slice(data).unwrap()
//     }
//
//     /// Deserializes a string slice into a Packet
//     // Is this ok?
//     #[allow(clippy::should_implement_trait)]
//     pub fn from_str(data: &str) -> Packet {
//         serde_json::from_str(data).unwrap()
//     }
// }
//
// /// A trait to be implemented on anything that can be converted into a Packet or
// /// from a Packet
// pub trait Packetize {
//     type Packets;
//     type PacketBytes;
//     type FlatPackets;
//     type PacketMap;
//     fn into_packets(self) -> Self::Packets;
//     fn as_packet_bytes(&self) -> Self::PacketBytes;
//     fn assemble(map: &mut Self::PacketMap) -> Self::FlatPackets;
//     fn try_assemble(map: &mut Self::PacketMap) -> StdResult<Self::FlatPackets, NotCompleteError>;
// }
//
// /// Required to use `NotCompleteError` as an Error type in the Result enum
// impl Error for NotCompleteError {}
//
// /// Required to use `NotCompleteError` as an Error type in the Result enum
// impl std::fmt::Display for NotCompleteError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "NotCompleteError")
//     }
// }
