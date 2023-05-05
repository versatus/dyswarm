pub mod broadcast_engine;
pub mod cache;
pub mod engine;
pub mod forwarder;
pub mod reassembler;
pub mod receiver;
pub mod utils;

pub use utils::*;

#[cfg(test)]
mod tests {
    //     use super::engine::*;
    //     use bytes::Bytes;
    //     use std::net::{Ipv6Addr, SocketAddr};
    //
    //     use crate::server::engine::{BroadcastEngine, Timeout};
    //     use crate::types::message::Message;
    //
    //     #[tokio::test]
    //     async fn test_successful_connection() {
    //         let mut b1 = BroadcastEngine::new(1234, 1145).await.unwrap();
    //         let mut b2 = BroadcastEngine::new(1235, 1145).await.unwrap();
    //
    //         let _ = b1
    //             .add_peer_connection(vec![SocketAddr::new(
    //                 std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
    //                 1235,
    //             )])
    //             .await;
    //
    //         let _ = b2
    //             .add_peer_connection(vec![SocketAddr::new(
    //                 std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
    //                 1234,
    //             )])
    //             .await;
    //
    //         if let Some((connection, _)) = b1.endpoint.1.next().await {
    //             assert_eq!(connection.remote_address(), b2.endpoint.0.public_addr());
    //         } else {
    //             panic!("No incoming connection");
    //         }
    //
    //         let _ = b1.remove_peer_connection(vec![SocketAddr::new(
    //             std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
    //             1234,
    //         )]);
    //         let _ = b1.remove_peer_connection(vec![SocketAddr::new(
    //             std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
    //             1235,
    //         )]);
    //     }
    //
    //     #[tokio::test]
    //     async fn test_broadcast_message_to_peers() {
    //         let mut b1 = BroadcastEngine::new(1236, 1145).await.unwrap();
    //         let mut b2 = BroadcastEngine::new(1237, 1145).await.unwrap();
    //         let mut b3 = BroadcastEngine::new(1238, 1145).await.unwrap();
    //
    //         let _ = b1
    //             .add_peer_connection(vec![SocketAddr::new(
    //                 std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
    //                 1237,
    //             )])
    //             .await;
    //
    //         let _ = b1
    //             .add_peer_connection(vec![SocketAddr::new(
    //                 std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
    //                 1238,
    //             )])
    //             .await;
    //
    //         let tst_msg = test_message();
    //         let _ = b1.quic_broadcast(tst_msg.clone()).await;
    //
    //         let _ = b1.quic_broadcast(tst_msg.clone()).await;
    //
    //         // Peer 2 gets an incoming connection
    //         let mut peer2_incoming_messages =
    //             if let Some((_, incoming)) = b2.get_incoming_connections().next().await {
    //                 incoming
    //             } else {
    //                 panic!("No incoming connection");
    //             };
    //
    //         if let Ok(message) = peer2_incoming_messages.next().timeout().await.unwrap() {
    //             assert_eq!(
    //                 message,
    //                 Some((Bytes::new(), Bytes::new(), Bytes::from(tst_msg.as_bytes())))
    //             );
    //         }
    //
    //         // Peer 2 gets an incoming connection
    //         let mut peer3_incoming_messages =
    //             if let Some((_, incoming)) = b3.get_incoming_connections().next().await {
    //                 incoming
    //             } else {
    //                 panic!("No incoming connection");
    //             };
    //
    //         if let Ok(message) = peer3_incoming_messages.next().timeout().await.unwrap() {
    //             assert_eq!(
    //                 message,
    //                 Some((Bytes::new(), Bytes::new(), Bytes::from(tst_msg.as_bytes())))
    //             );
    //         }
    //     }
    //
    //     pub fn test_message() -> Message {
    //         let msg = Message {
    //             id: uuid::Uuid::new_v4(),
    //             data: MessageBody::Empty,
    //             _marker: std::marker::PhantomData,
    //         };
    //         msg
    //     }
}
