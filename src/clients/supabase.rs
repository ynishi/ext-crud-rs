use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use postgrest::Postgrest;
use serde::Serialize;

use crate::traits::Client;

pub struct SupabaseClient {
    pub postgrest: Postgrest,
}

impl SupabaseClient {
    pub fn new(url: &str) -> Self {
        let postgrest = Postgrest::new(url);
        Self { postgrest }
    }
}

#[async_trait]
impl Client for SupabaseClient {
    async fn create<T: Serialize + Send + Sync>(&self, table: &str, item: &T) -> Result<()> {
        let tag = "SupabaseClient.create";
        let s = serde_json::to_string(item).map_err(|e| anyhow!(e).context(tag))?;

        let client = self.postgrest.clone();

        let response = client
            .from(table)
            .insert(s)
            .execute()
            .await
            .map_err(|e| anyhow!(e).context(tag))?;

        if !response.status().is_success() {
            bail!("{}, Request failed with status: {}", tag, response.status())
        }
        Ok(())
    }

    async fn find_by_keys<K: Serialize + Send + Sync>(
        &self,
        table: &str,
        key: &str,
        ids: Vec<K>,
    ) -> Result<Vec<serde_json::Value>> {
        let tag = "SupabaseClient.find_by_keys";

        let client = self.postgrest.clone();
        let ids = ids
            .iter()
            .map(|id| serde_json::to_string(id).map_err(|e| anyhow!(e).context(tag)))
            .collect::<Result<Vec<String>>>()?;
        let response = client.from(table).in_(key, &ids).execute().await?;
        if !response.status().is_success() {
            bail!(format!(
                "{}, Request failed with status: {}",
                tag,
                response.status()
            ));
        }
        let text = response.text().await.map_err(|e| anyhow!(e).context(tag))?;
        let data = serde_json::from_str(&text).map_err(|e| anyhow!(e).context(tag))?;
        Ok(data)
    }

    async fn update_by_keys<K: Serialize + Send + Sync, T: Serialize + Send + Sync>(
        &self,
        table: &str,
        key: &str,
        items: Vec<(K, T)>,
    ) -> Result<()> {
        let tag = "SupabaseClient.update_by_keys";

        let client = self.postgrest.clone();
        for item in items {
            let mut query = client
                .from(table)
                .update(serde_json::to_string(&item.1).map_err(|e| anyhow!(e).context(tag))?);
            let id = serde_json::to_string(&item.0).map_err(|e| anyhow!(e).context(tag))?;
            query = query.eq(key, &id);
            let response = query.execute().await?;
            if !response.status().is_success() {
                bail!(format!(
                    "{}, Request failed with status: {}",
                    tag,
                    response.status()
                ));
            }
        }
        Ok(())
    }

    async fn delete_by_keys<K: Serialize + Send + Sync>(
        &self,
        table: &str,
        key: &str,
        ids: Vec<K>,
    ) -> Result<()> {
        let tag = "SupabaseClient.delete_by_keys";

        let client = self.postgrest.clone();
        for id in ids {
            let id = serde_json::to_string(&id).map_err(|e| anyhow!(e).context(tag))?;
            let mut query = client.from(table).delete();
            query = query.eq(key, id);
            let response = query.execute().await?;
            if !response.status().is_success() {
                bail!(format!(
                    "{}, Request failed with status: {}",
                    tag,
                    response.status()
                ));
            }
        }
        Ok(())
    }
}
