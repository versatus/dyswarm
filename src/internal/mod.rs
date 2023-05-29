pub mod cache;
pub mod engine;
pub mod forwarder;
pub mod reassembler;
pub mod receiver;
pub mod utils;

pub use engine::*;
pub use utils::*;

#[cfg(test)]
mod tests {
    use super::engine::*;
    use bytes::Bytes;

    use crate::internal::{Engine, Timeout};
    use crate::types::message::Message;

    #[tokio::test]
    async fn test_successful_connection() {
        let engine_config_1 = EngineConfig::default();
        let engine_config_2 = EngineConfig::default();

        let mut peer_1 = Engine::new(engine_config_1).await.unwrap();
        let mut peer_2 = Engine::new(engine_config_2).await.unwrap();

        let addr_1 = peer_1.public_addr();
        let addr_2 = peer_2.public_addr();

        peer_1.add_peer_connections(vec![addr_2]).await.unwrap();
        peer_2.add_peer_connections(vec![addr_1]).await.unwrap();

        let (connection, _) = peer_1.get_incoming_connections().next().await.unwrap();
        assert_eq!(connection.remote_address(), peer_2.public_addr());

        peer_1.remove_peer_connections(vec![addr_2]).unwrap();
        peer_2.remove_peer_connections(vec![addr_1]).unwrap();
    }

    #[tokio::test]
    async fn test_broadcast_message_to_peers() {
        let engine_config_1 = EngineConfig::default();
        let engine_config_2 = EngineConfig::default();
        let engine_config_3 = EngineConfig::default();

        let mut peer_1 = Engine::new(engine_config_1).await.unwrap();

        let mut peer_2 = Engine::new(engine_config_2).await.unwrap();
        let addr_2 = peer_2.public_addr();

        let mut peer_3 = Engine::new(engine_config_3).await.unwrap();
        let addr_3 = peer_3.public_addr();

        peer_1
            .add_peer_connections(vec![addr_2, addr_3])
            .await
            .unwrap();

        let test_msg = test_message();
        let test_bytes = Bytes::from(test_message().as_bytes());

        peer_1.quic_broadcast(test_bytes.clone()).await.unwrap();

        let (_, mut incoming_rx) = peer_2.get_incoming_connections().next().await.unwrap();
        let message = incoming_rx
            .next()
            .timeout()
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert_eq!(message, (Bytes::new(), Bytes::new(), test_bytes.clone()));

        let message_deser = Message::from(message.2);
        assert_eq!(message_deser, test_msg);

        let (_, mut incoming_rx) = peer_3.get_incoming_connections().next().await.unwrap();
        let message = incoming_rx
            .next()
            .timeout()
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert_eq!(message, (Bytes::new(), Bytes::new(), test_bytes.clone()));

        let message_deser = Message::from(message.2);
        assert_eq!(message_deser, test_msg);
    }

    pub fn test_message() -> Message<f32> {
        Message {
            id: uuid::Uuid::nil(),
            timestamp: 0,
            data: 3.141599,
        }
    }
}
