use crate::types::constants::BATCH_ID_SIZE;
use crate::types::PAYLOAD_SIZE;

use rand::{distributions::Alphanumeric, thread_rng, Rng, RngCore};
use raptorq::Encoder;
use tracing::debug;

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
