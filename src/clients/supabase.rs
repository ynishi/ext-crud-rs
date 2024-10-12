use crate::traits::Client;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use log::debug;
use postgrest::Postgrest;
use serde::Serialize;

pub struct SupabaseClient {
    pub postgrest: Postgrest,
}

impl SupabaseClient {
    pub fn new(url: &str, key: &str) -> Self {
        let postgrest = Self::new_postgrest(url, key);
        Self { postgrest }
    }

    pub(crate) fn new_postgrest(url: &str, key: &str) -> Postgrest {
        let endpoint = format!("{}/rest/v1/", url);
        Postgrest::new(endpoint)
            .insert_header("apikey", key)
            .insert_header("Authorization", format!("Bearer {}", key))
    }
}

#[async_trait]
impl Client for SupabaseClient {
    async fn create<T: Serialize + Send + Sync>(&self, table: &str, item: &T) -> Result<()> {
        let tag = "SupabaseClient.create";
        debug!("SupabaseClient.create: {}, table: {}", tag, table);
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
    ) -> Result<()>
    where
        K: ToString + AsRef<str>,
    {
        let tag = "SupabaseClient.update_by_keys";

        let client = self.postgrest.clone();
        for item in items {
            println!("{}", serde_json::to_string(&item.1).unwrap());
            let mut query = client
                .from(table)
                .update(serde_json::to_string(&item.1).map_err(|e| anyhow!(e).context(tag))?);
            query = query.eq(key, item.0);

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
