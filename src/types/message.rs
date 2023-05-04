use bytes::Bytes;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use crate::types::constants::MessageId;

/// The Message struct contains the basic data contained in a message
/// sent across the network. This can be packed into bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<D>
where
    D: Default + Debug + Clone,
{
    pub id: MessageId,
    pub data: D,
}

impl<D> Default for Message<D>
where
    D: Default + Debug + Clone,
{
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
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
        Message {
            id: Uuid::new_v4(),
            data,
        }
    }

    /// Returns an empty, nil message
    pub fn nil() -> Message<D> {
        Message {
            id: uuid::Uuid::nil(),
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

#[cfg(test)]
mod tests {
    fn test_message() {
        let msg = super::Message::new("Hello World");
        let bytes: Vec<u8> = msg.into();
        let msg: super::Message<String> = bytes.into();
        assert_eq!(msg.data, "Hello World");
    }
}
