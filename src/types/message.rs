use bytes::Bytes;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use crate::types::constants::MessageId;

/// The Message struct contains the basic data contained in a message
/// sent across the network. This can be packed into bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message<D>
where
    D: Default + Debug + Clone,
{
    pub id: MessageId,
    pub timestamp: i64,
    pub data: D,
}

impl<D> Default for Message<D>
where
    D: Default + Debug + Clone,
{
    fn default() -> Self {
        let timestamp = chrono::offset::Utc::now().timestamp();
        Self {
            id: Uuid::nil(),
            timestamp,
            data: Default::default(),
        }
    }
}

/// AsMessage is a trait that when implemented on a custom type allows
/// for the easy conversion of the type into a message that can be packed
/// into a byte array and sent across the network.
pub trait AsMessage<D>
where
    D: Default + Debug + Clone,
{
    fn into_message(self, return_receipt: u8) -> Message<D>;
}

impl<'a, D> Message<D>
where
    D: Default + Debug + Clone + Serialize + Deserialize<'a>,
{
    /// Generate a new Message, identified with an UUID v4
    pub fn new(data: D) -> Self {
        let timestamp = chrono::offset::Utc::now().timestamp();
        Message {
            id: Uuid::new_v4(),
            timestamp,
            data,
        }
    }

    /// Returns an empty, nil message
    pub fn nil() -> Message<D> {
        Message {
            id: uuid::Uuid::nil(),
            timestamp: 0,
            data: D::default(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.clone().into()
    }
}

impl<D> From<Vec<u8>> for Message<D>
where
    D: Default + Debug + Clone + DeserializeOwned,
{
    fn from(data: Vec<u8>) -> Self {
        bincode::deserialize(&data).unwrap_or_default()
    }
}

impl<D> From<Message<D>> for Vec<u8>
where
    D: Default + Debug + Clone + Serialize,
{
    fn from(msg: Message<D>) -> Self {
        bincode::serialize(&msg).unwrap_or_default()
    }
}

impl<D> From<Message<D>> for Bytes
where
    D: Default + Debug + Clone + Serialize,
{
    fn from(msg: Message<D>) -> Self {
        let msg: Vec<u8> = msg.into();
        Bytes::from(msg)
    }
}

impl<D> From<Bytes> for Message<D>
where
    D: Default + Debug + Clone + Serialize + DeserializeOwned,
{
    fn from(bytes: Bytes) -> Self {
        let vec_bytes = bytes.to_vec();
        Message::from(vec_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn test_message_serde() {
        let msg = Message::new("Hello World");
        let bytes: Vec<u8> = msg.into();
        let msg: Message<String> = bytes.into();
        assert_eq!(msg.data, "Hello World");
    }
}
