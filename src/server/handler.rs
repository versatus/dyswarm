use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{types::Message, Result};

#[async_trait]
pub trait Handler<D>
where
    D: std::fmt::Debug + Default + Serialize + DeserializeOwned + Clone,
{
    async fn handle(&self, msg: Message<D>) -> Result<()>;
}
