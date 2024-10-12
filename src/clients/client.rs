use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

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
