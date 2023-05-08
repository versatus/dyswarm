use crossbeam_channel::Receiver;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::task::JoinSet;
use tracing::{error, info};

/// It receives a packet from the `forwarder_channel_rx` channel, clones
/// it, and sends it to all the nodes in the network except itself
///
/// Arguments:
///
/// * `forwarder_channel_rx`: Receiver<Vec<u8>>
/// * `nodes_ips_except_self`: This is a vector of IP addresses of all the nodes
///   in the network except
/// the current node.
/// * `port`: The port to bind the socket to.
///
/// Returns:
///
/// A future that will be executed when the packet_forwarder function is called.
pub async fn packet_forwarder(
    forwarder_channel_receive: Receiver<Vec<u8>>,
    nodes_ips_except_self: Vec<Vec<u8>>,
    port: u16,
    udp_socket: UdpSocket,
) -> crate::types::Result<()> {
    let udp_socket = Arc::new(udp_socket);

    loop {
        let nodes = nodes_ips_except_self.clone();
        match forwarder_channel_receive.recv() {
            Ok(packet) => {
                info!("bytes received: {}", packet.len());

                let mut broadcast_futures = JoinSet::new();

                for addr in nodes {
                    let pack = packet.clone();
                    let sock = udp_socket.clone();

                    broadcast_futures.spawn(async move {
                        sock.send_to(&pack, (&String::from_utf8_lossy(&addr)[..], port))
                            .await
                    });
                }
                while broadcast_futures.join_next().await.is_some() {
                    // log result
                }
            }
            Err(e) => {
                error!("Error occurred while receiving packet: {:?}", e)
            }
        }
    }
}
