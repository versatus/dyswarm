use std::net::SocketAddr;
use std::sync::Arc;
use std::{collections::HashSet, io::Read, str};

// use block::Block;
use crate::engine::cache::Cache;
use crate::types::constants::BATCH_ID_SIZE;
use crate::types::constants::NUM_RCVMMSGS;
use crate::types::PAYLOAD_SIZE;
use crate::types::{Result, DECODER_DATA_INDEX};
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use rand::{distributions::Alphanumeric, thread_rng, Rng, RngCore};
use raptorq::{Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

use super::{get_batch_id, get_payload_length, get_symbol_size};

/// This function receives packets, decodes them using a RaptorQ decoder, and
/// forwards the decoded data to a batch send channel.
///
/// Arguments:
///
/// * `receiver`: A `Receiver` that receives tuples of a byte array of size 1280
///   and a usize value.
/// * `batch_id_hashset`: A mutable reference to a HashSet that stores the batch
///   IDs of packets that
/// have already been received and processed.
/// * `decoder_hash_cache`: `decoder_hash_cache` is a mutable reference to a
///   `Cache` data structure that
/// maps a batch ID (represented as a `[u8; BATCH_ID_SIZE]` array) to a tuple
/// containing the number of packets received so far for that batch and a
/// `Decoder` object.
/// * `forwarder`: `forwarder` is a `Sender` channel used to forward packets to
///   another node in the
/// network. It is used when a packet has a forward flag set to 1, indicating
/// that it needs to be forwarded to another node. The `forwarder` channel
/// allows the reassembled packets to
/// * `batch_send`: `batch_send` is a `Sender` channel used to send decoded data
///   to another part of the
/// program. Specifically, it is used to send `RaptorBroadCastedData` structs
/// that have been deserialized from received packets.
pub fn reassemble_packets(
    receiver: Receiver<([u8; 1280], usize)>,
    batch_id_hashset: &mut HashSet<[u8; BATCH_ID_SIZE]>,
    decoder_hash_cache: &mut Cache<[u8; BATCH_ID_SIZE], (usize, Decoder)>,
    forwarder: Sender<Vec<u8>>,
    batch_send: Sender<Bytes>,
) {
    loop {
        let mut received_packet = match receiver.recv() {
            Ok(pr) => pr,
            Err(_e) => {
                continue;
            }
        };

        let batch_id = get_batch_id(&received_packet.0);

        if batch_id_hashset.contains(&batch_id) {
            continue;
        }

        let payload_length = get_payload_length(&received_packet.0);
        let symbol_size = get_symbol_size(&received_packet.0);

        // This is to check if the packet is a forwarder packet. If it is, it forwards
        // the packet to the `forwarder` channel. Since packet is shared across
        // nodes with forward flag as 1
        if let Some(forward_flag) = received_packet.0.get_mut(1) {
            if *forward_flag == 1 {
                *forward_flag = 0;
                let _ = forwarder
                    .try_send(received_packet.0[DECODER_DATA_INDEX..received_packet.1].to_vec());
            }
        }

        match decoder_hash_cache.get_mut(&batch_id) {
            Some((num_packets, decoder)) => {
                *num_packets += 1;
                // Decoding the packet.
                let result = decoder.decode(EncodingPacket::deserialize(
                    &received_packet.0[40_usize..received_packet.1],
                ));
                if result.is_some() {}
                if let Some(result_bytes) = result {
                    batch_id_hashset.insert(batch_id);
                    if let Ok(batch_id_str) = str::from_utf8(&batch_id) {
                        let batch_id_str = String::from(batch_id_str);
                        let msg = (batch_id_str, result_bytes);
                        if let Ok(data) = String::from_utf8(msg.1.clone()) {
                            let data = data.trim_end_matches('\0').to_string().replace("\\", "");
                            match serde_json::from_str::<Bytes>(&data) {
                                Ok(data) => {
                                    let _ = batch_send.send(data);
                                }
                                Err(e) => {
                                    error!(
                                        "Error occured while unmarshalling  :{:?}",
                                        e.to_string()
                                    );
                                }
                            }
                        }
                        decoder_hash_cache.remove(&batch_id);
                    }
                }
            }
            None => {
                // This is creating a new decoder for a new batch.
                decoder_hash_cache.push(
                    batch_id,
                    (
                        1_usize,
                        Decoder::new(ObjectTransmissionInformation::new(
                            payload_length as u64,
                            symbol_size as u16,
                            1,
                            1,
                            8,
                        )),
                    ),
                );
            }
        }
    }
}
