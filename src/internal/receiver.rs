use std::net::SocketAddr;

use crate::types::constants::NUM_RCVMMSGS;
use crate::types::Result;
use tokio::net::UdpSocket;
use tracing::error;

// NOTE: For Linux we can use system call from libc::recv_mmsg

/// It receives a UDP packet from a socket, and
/// returns the index of the packet in the array, the number of bytes received,
/// and the address of the sender
///
/// Arguments:
///
/// * `socket`: The UDP socket to receive from.
/// * `packets`: a mutable array of byte arrays, each of which is the size of
///   the largest packet you
/// want to receive.
//#[cfg(not(target_os = "linux"))]
pub async fn recv_mmsg(
    socket: &UdpSocket,
    packets: &mut [[u8; 1280]],
) -> Result<Vec<(usize, usize, SocketAddr)>> {
    let mut received = Vec::new();

    let count = std::cmp::min(NUM_RCVMMSGS, packets.len());

    for (i, packt) in packets.iter_mut().take(count).enumerate() {
        match socket.recv_from(packt).await {
            Err(err) => {
                error!("Error occured while receiving packet :{:?}", err);
            }
            Ok((nrecv, from)) => {
                received.push((i, nrecv, from));
            }
        }
    }

    Ok(received)
}

#[derive(Debug)]
pub struct PacketReceiver {
    //
}

impl PacketReceiver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run() {
        //
    }
}
