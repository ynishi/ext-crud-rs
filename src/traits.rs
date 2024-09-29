use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait Client: Send + Sync + 'static {
    async fn create<T: Serialize + Send + Sync>(
        &self,
        table: &str,
        item: &T,
    ) -> Result<serde_json::Value>;

    async fn find_by_ids<K: Serialize + Send + Sync>(
        &self,
        table: &str,
        ids: Vec<K>,
    ) -> Result<Vec<serde_json::Value>>;
}

#[async_trait]
pub trait ExtendedCrud<C: Client>:
    Sized + Serialize + DeserializeOwned + Send + Sync + 'static
{
    type PrimaryKey: Serialize + DeserializeOwned + Send + Sync + 'static;

    const TABLE_NAME: &'static str;

    fn to_entity(value: serde_json::Value) -> Result<Self> {
        serde_json::from_value(value).map_err(|e| anyhow!(e))
    }

    async fn create(self, client: &C) -> Result<Self> {
        let value = client.create(Self::TABLE_NAME, &self).await?;
        Self::to_entity(value)
    }

    async fn read(id: Self::PrimaryKey, client: &C) -> Result<Self> {
        let value = client
            .find_by_ids::<Self::PrimaryKey>(Self::TABLE_NAME, vec![id])
            .await?
            .pop()
            .ok_or_else(|| anyhow!("Not found"))?;
        Self::to_entity(value)
    }

    async fn read_many(ids: Vec<Self::PrimaryKey>, client: &C) -> Result<Vec<Self>> {
        let values = client
            .find_by_ids::<Self::PrimaryKey>(Self::TABLE_NAME, ids)
            .await?;
        values.into_iter().map(Self::to_entity).collect()
    }

    fn primary_key(&self) -> &Self::PrimaryKey;
}
