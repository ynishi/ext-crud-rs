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
    async fn create<T: Serialize + Send + Sync>(
        &self,
        table: &str,
        item: &T,
    ) -> Result<serde_json::Value> {
        let value = serde_json::json!(item).to_string();
        let response = self
            .postgrest
            .from(table)
            .insert(value)
            .execute()
            .await
            .map_err(|e| anyhow!(e))?;
        if !response.status().is_success() {
            bail!("Request failed with status: {}", response.status())
        }
        let txt = response.text().await?;
        let data_list: Vec<serde_json::Value> = serde_json::from_str(&txt)?;
        Ok(data_list.first().unwrap().clone())
    }

    async fn find_by_ids<K: Serialize + Send + Sync>(
        &self,
        table: &str,
        ids: Vec<K>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut query = String::new();
        for id in ids {
            let id_str = serde_json::json!(&id);
            query.push_str(&format!("id=eq.{}&", id_str));
        }
        let response = self
            .postgrest
            .from(table)
            .select("*")
            .eq("id", &query)
            .execute()
            .await
            .map_err(|e| anyhow!(e))?;
        if !response.status().is_success() {
            bail!("Request failed with status: {}", response.status())
        }
        let txt = response.text().await?;
        let data_list: Vec<serde_json::Value> = serde_json::from_str(&txt)?;
        Ok(data_list)
    }
}
