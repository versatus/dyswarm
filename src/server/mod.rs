pub mod handler;
pub mod server;

pub use handler::*;
pub use server::*;

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{
        client::{Client, Config},
        internal::{Engine, EngineConfig},
        types::Message,
        Result,
    };

    use super::*;

    #[tokio::test]
    async fn should_handle_connections() {
        let client_config = Config::default();
        let client = Client::new(client_config).await.unwrap();

        let server = Server::new().await.unwrap();
        let addr_1 = server.public_addr();

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        let handler = new_test_handler(tx);
        let handle = server.run(handler).await.unwrap();

        let message = Message::new(42);

        client
            .send_data_via_quic(message.clone(), addr_1)
            .await
            .unwrap();

        let mess = rx.recv().await.unwrap();

        assert_eq!(mess, message);

        handle.stop().await;
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
