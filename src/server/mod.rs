pub mod handler;
pub mod server;

pub use handler::*;
pub use server::*;

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{
        engine::{Engine, EngineConfig},
        types::Message,
        Result,
    };

    use super::*;

    #[tokio::test]
    async fn should_handle_connections() {
        let engine_config_1 = EngineConfig::default();
        let engine_config_2 = EngineConfig::default();

        let engine_1 = Engine::new(engine_config_1).await.unwrap();
        let engine_2 = Engine::new(engine_config_2).await.unwrap();

        let addr_1 = engine_1.public_addr();

        let server = Server::new(engine_1);

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        let handler = new_test_handler(tx);
        let handle = server.run(handler).await.unwrap();

        let message = Message::new(42);
        let message_bytes = message.clone().into();

        engine_2
            .send_data_via_quic(message_bytes, addr_1)
            .await
            .unwrap();

        let mess = rx.recv().await.unwrap();

        assert_eq!(mess, message);

        handle.abort();
    }

    struct HandlerImpl {
        pub tx: tokio::sync::mpsc::Sender<Message<i32>>,
    }

    #[async_trait]
    impl Handler<i32> for HandlerImpl {
        async fn handle(&self, msg: Message<i32>) -> Result<()> {
            self.tx.send(msg).await.unwrap();
            Ok(())
        }
    }

    fn new_test_handler(tx: tokio::sync::mpsc::Sender<Message<i32>>) -> HandlerImpl {
        HandlerImpl { tx }
    }
}
