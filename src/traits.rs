use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait Client: Send + Sync + 'static {
    async fn create<T: Serialize + Send + Sync>(&self, table: &str, item: &T) -> Result<()>;

    async fn find_by_keys<K: Serialize + Send + Sync>(
        &self,
        table: &str,
        key: &str,
        ids: Vec<K>,
    ) -> Result<Vec<serde_json::Value>>;

    async fn update_by_keys<K: Serialize + Send + Sync, T: Serialize + Send + Sync>(
        &self,
        table: &str,
        key: &str,
        items: Vec<(K, T)>,
    ) -> Result<()>
    where
        K: ToString + std::convert::AsRef<str>;

    async fn delete_by_keys<K: Serialize + Send + Sync>(
        &self,
        table: &str,
        key: &str,
        ids: Vec<K>,
    ) -> Result<()>;

    fn as_str<T: Serialize>(&self, v: T) -> String {
        serde_json::json!(v).to_string()
    }
}

#[async_trait]
pub trait ExtendedCrud<C: Client>:
    Sized + Serialize + DeserializeOwned + Send + Sync + 'static
{
    type PrimaryKey: Serialize + DeserializeOwned + Send + Sync + 'static + ToString;

    const TABLE_NAME: &'static str;

    const PRIMARY_KEY_NAME: &'static str;

    fn to_entity(value: serde_json::Value) -> Result<Self> {
        serde_json::from_value(value).map_err(|e| anyhow!(e))
    }

    async fn create(self, client: &C) -> Result<()> {
        client
            .create(Self::TABLE_NAME, &self)
            .await
            .map_err(|e| anyhow!(e).context("ExtendedCrud.create failed"))
    }

    async fn read(client: &C, id: Self::PrimaryKey) -> Result<Self> {
        let tag = "ExtendedCrud.read failed";
        let mut founds = client
            .find_by_keys::<Self::PrimaryKey>(Self::TABLE_NAME, Self::PRIMARY_KEY_NAME, vec![id])
            .await
            .context(tag)?;
        if founds.len() > 1 {
            anyhow::bail!(format!("{}, Found more than one", tag));
        }
        let value = founds
            .pop()
            .ok_or_else(|| anyhow!("Not found").context(tag))?;
        Self::to_entity(value)
    }

    async fn read_many(ids: Vec<Self::PrimaryKey>, client: &C) -> Result<Vec<Self>> {
        let tag = "ExtendedCrud.read_many failed";
        let founds = client
            .find_by_keys::<Self::PrimaryKey>(Self::TABLE_NAME, Self::PRIMARY_KEY_NAME, ids)
            .await
            .context(tag)?;
        founds.into_iter().map(Self::to_entity).collect()
    }

    async fn update(&self, client: &C) -> Result<()> {
        let tag = "ExtendedCrud.update failed";
        client
            .update_by_keys(
                Self::TABLE_NAME,
                Self::PRIMARY_KEY_NAME,
                vec![(self.primary_key().to_string(), &self)],
            )
            .await
            .context(tag)
    }

    async fn update_many(items: Vec<Self>, client: &C) -> Result<()> {
        let tag = "ExtendedCrud.update_many failed";
        let items = items
            .into_iter()
            .map(|e| (client.as_str(e.primary_key()), e))
            .collect();
        client
            .update_by_keys(Self::TABLE_NAME, Self::PRIMARY_KEY_NAME, items)
            .await
            .context(tag)
    }

    async fn delete(self, client: &C) -> Result<()> {
        let tag = "ExtendedCrud.delete failed";
        let id = client.as_str(self.primary_key());
        client
            .delete_by_keys(Self::TABLE_NAME, Self::PRIMARY_KEY_NAME, vec![id])
            .await
            .context(tag)
    }

    async fn delete_many(ids: Vec<Self::PrimaryKey>, client: &C) -> Result<()> {
        let tag = "ExtendedCrud.delete_many failed";
        let ids = ids.into_iter().map(|e| client.as_str(e)).collect();
        client
            .delete_by_keys(Self::TABLE_NAME, Self::PRIMARY_KEY_NAME, ids)
            .await
            .context(tag)
    }

    fn primary_key(&self) -> &Self::PrimaryKey;
}

pub trait PartialStruct<T>: Serialize + Send + Sync + 'static {
    type PrimaryKey: Serialize + Send + Sync + 'static + ToString;

    const PRIMARY_KEY_NAME: &'static str;

    fn new() -> Self;

    fn apply_to(&self, original: &T) -> T;

    fn primary_key(&self) -> Option<Self::PrimaryKey>;
}
