use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use log::debug;
use postgrest::Postgrest;
use serde::Serialize;

use crate::clients::client::Client;
use crate::entity::query::{Query, QueryField};

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
    async fn creates<T: Serialize + Send + Sync>(&self, table: &str, items: Vec<T>) -> Result<()> {
        let tag = "SupabaseClient.create";
        debug!("SupabaseClient.create: {}, table: {}", tag, table);
        for item in items {
            let s = serde_json::to_string(&item).map_err(|e| anyhow!(e).context(tag))?;
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
        }
        Ok(())
    }

    async fn list(&self, table: &str) -> Result<Vec<serde_json::Value>> {
        let tag = "SupabaseClient.list";

        let client = self.postgrest.clone();
        let response = client.from(table).select("*").execute().await?;
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

    async fn find<T: QueryField>(
        &self,
        table: &str,
        _query: &Query<T>,
    ) -> Result<Vec<serde_json::Value>> {
        let tag = "SupabaseClient.find";
        let client = self.postgrest.clone();

        // TODO: Implement query facade
        // let query = serde_json::to_value(query).map_err(|e| anyhow!(e).context(tag))?;
        let query = serde_json::to_value(1).map_err(|e| anyhow!(e).context(tag))?;
        let response = client
            .from(table)
            .select(query.to_string())
            .execute()
            .await?;
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

    async fn count(&self, table: &str) -> Result<u64> {
        let tag = "SupabaseClient.count";

        let client = self.postgrest.clone();
        let response = client.from(table).select("count(*)").execute().await?;
        if !response.status().is_success() {
            bail!(format!(
                "{}, Request failed with status: {}",
                tag,
                response.status()
            ));
        }
        let text = response.text().await.map_err(|e| anyhow!(e).context(tag))?;
        let data: u64 = serde_json::from_str(&text).map_err(|e| anyhow!(e).context(tag))?;
        Ok(data)
    }
}
/*
pub struct PostgrestQueryBuilder<'a> {
    client: &'a Postgrest,
    table_name: &'a str,
}

impl<'a> PostgrestQueryBuilder<'a> {
    pub fn new(client: &'a Postgrest, table_name: &'a str) -> Self {
        Self { client, table_name }
    }

    pub fn build(&self, query: &Query) -> Result<postgrest::Builder> {
        let mut builder = self.client.from(self.table_name).select("*");

        // ページネーションの適用
        if let Some(pagination) = &query.pagenation {
            if let (Some(current), Some(page_size)) = (pagination.current, pagination.page_size) {
                let start = ((current - 1) * page_size) as usize;
                let end = (current * page_size - 1) as usize;
                builder = builder.range(start, end);
            }
        }

        // ソート順の適用
        if let Some(sorters) = &query.sorters {
            for sorter in sorters {
                let ascending = match sorter.order {
                    SortOrder::Ascending => true,
                    SortOrder::Descending => false,
                };
                let rel: Option<&str> = None;
                builder = builder.order_with_options(&sorter.field, rel, ascending, false);
            }
        }

        // フィルターの適用
        if let Some(filters) = &query.filters {
            for filter in filters {
                builder = self.apply_filter(builder, filter)?;
            }
        }

        Ok(builder)
    }

    fn apply_filter(
        &self,
        builder: postgrest::Builder,
        filter: &Filter,
    ) -> Result<postgrest::Builder> {
        match filter {
            Filter::LogicalFilter {
                field,
                operator,
                value,
            } => self.apply_logical_filter(builder, field, operator, value),
            Filter::ConditionalFilter {
                key: _key,
                operator,
                filters,
            } => self.apply_conditional_filter(builder, operator, filters),
        }
    }

    fn apply_logical_filter(
        &self,
        builder: postgrest::Builder,
        field: &str,
        operator: &LogicalFilterOperator,
        value: &serde_json::Value,
    ) -> Result<postgrest::Builder> {
        let builder = match operator {
            LogicalFilterOperator::Eq => builder.eq(field, value.to_string()),
            LogicalFilterOperator::Ne => builder.neq(field, value.to_string()),
            LogicalFilterOperator::Lt => builder.lt(field, value.to_string()),
            LogicalFilterOperator::Gt => builder.gt(field, value.to_string()),
            LogicalFilterOperator::Lte => builder.lte(field, value.to_string()),
            LogicalFilterOperator::Gte => builder.gte(field, value.to_string()),
            LogicalFilterOperator::In => {
                if let serde_json::Value::Array(arr) = value {
                    let str_values: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                    builder.in_(field, str_values)
                } else {
                    bail!("Value must be an array for In operator");
                }
            }
            LogicalFilterOperator::Nin => builder.not("in", field, value.to_string()),
            LogicalFilterOperator::Contains | LogicalFilterOperator::Containss => builder.ilike(
                field,
                format!(
                    "%{}%",
                    value.as_str().ok_or(anyhow!("Value must be string"))?
                ),
            ),
            LogicalFilterOperator::Ncontains | LogicalFilterOperator::Ncontainss => builder.not(
                "ilike",
                field,
                format!(
                    "%{}%",
                    value.as_str().ok_or(anyhow!("Value must be string"))?
                ),
            ),
            LogicalFilterOperator::Startswith | LogicalFilterOperator::Startswiths => builder
                .ilike(
                    field,
                    format!(
                        "{}%",
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    ),
                ),
            LogicalFilterOperator::Nstartswith | LogicalFilterOperator::Nstartswiths => builder
                .not(
                    "ilike",
                    field,
                    format!(
                        "{}%",
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    ),
                ),
            LogicalFilterOperator::Endswith | LogicalFilterOperator::Endswiths => builder.ilike(
                field,
                format!(
                    "%{}",
                    value.as_str().ok_or(anyhow!("Value must be string"))?
                ),
            ),
            LogicalFilterOperator::Nendswith | LogicalFilterOperator::Nendswiths => builder.not(
                "ilike",
                field,
                format!(
                    "%{}",
                    value.as_str().ok_or(anyhow!("Value must be string"))?
                ),
            ),
            LogicalFilterOperator::Between => {
                let array = value
                    .as_array()
                    .ok_or(anyhow!("Value must be array with two elements"))?;
                if array.len() != 2 {
                    bail!("Between operator requires exactly two values");
                }
                builder
                    .gte(field, array[0].to_string())
                    .lte(field, array[1].to_string())
            }
            LogicalFilterOperator::Nbetween => {
                let array = value
                    .as_array()
                    .ok_or(anyhow!("Value must be array with two elements"))?;
                if array.len() != 2 {
                    bail!("Between operator requires exactly two values");
                }
                builder.or(format!(
                    "{}.lt.{},or,{}.gt.{}",
                    field, array[0], field, array[1]
                ))
            }
            LogicalFilterOperator::Null => builder.is(field, "null"),
            LogicalFilterOperator::Nnull => builder.not("is", field, "null"),
            LogicalFilterOperator::Ina => builder.cs(field, value.to_string()),
            LogicalFilterOperator::Nina => builder.not("cs", field, value.to_string()),
        };

        Ok(builder)
    }

    fn apply_conditional_filter(
        &self,
        builder: postgrest::Builder,
        operator: &ConditionalFilterOperator,
        filters: &[Filter],
    ) -> Result<postgrest::Builder> {
        match operator {
            ConditionalFilterOperator::And => {
                let mut current_builder = builder;
                for filter in filters {
                    current_builder = self.apply_filter(current_builder, filter)?;
                }
                Ok(current_builder)
            }
            ConditionalFilterOperator::Or => {
                let conditions: Vec<String> = filters
                    .iter()
                    .map(Self::filter_to_string)
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(builder.or(conditions.join(",")))
            }
        }
    }

    fn filter_to_string(filter: &Filter) -> Result<String> {
        match filter {
            Filter::LogicalFilter {
                field,
                operator,
                value,
            } => Ok(match operator {
                LogicalFilterOperator::Eq => format!("{}.eq.{}", field, value),
                LogicalFilterOperator::Ne => format!("{}.neq.{}", field, value),
                LogicalFilterOperator::Lt => format!("{}.lt.{}", field, value),
                LogicalFilterOperator::Gt => format!("{}.gt.{}", field, value),
                LogicalFilterOperator::Lte => format!("{}.lte.{}", field, value),
                LogicalFilterOperator::Gte => format!("{}.gte.{}", field, value),
                LogicalFilterOperator::In => format!("{}.in.{}", field, value),
                LogicalFilterOperator::Nin => format!("{}.not.in.{}", field, value),
                LogicalFilterOperator::Contains | LogicalFilterOperator::Containss => {
                    format!(
                        "{}.ilike.*{}*",
                        field,
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    )
                }
                LogicalFilterOperator::Ncontains | LogicalFilterOperator::Ncontainss => {
                    format!(
                        "{}.not.ilike.*{}*",
                        field,
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    )
                }
                LogicalFilterOperator::Startswith | LogicalFilterOperator::Startswiths => {
                    format!(
                        "{}.ilike.{}*",
                        field,
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    )
                }
                LogicalFilterOperator::Nstartswith | LogicalFilterOperator::Nstartswiths => {
                    format!(
                        "{}.not.ilike.{}*",
                        field,
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    )
                }
                LogicalFilterOperator::Endswith | LogicalFilterOperator::Endswiths => {
                    format!(
                        "{}.ilike.*{}",
                        field,
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    )
                }
                LogicalFilterOperator::Nendswith | LogicalFilterOperator::Nendswiths => {
                    format!(
                        "{}.not.ilike.*{}",
                        field,
                        value.as_str().ok_or(anyhow!("Value must be string"))?
                    )
                }
                LogicalFilterOperator::Between => {
                    let array = value
                        .as_array()
                        .ok_or(anyhow!("Value must be array with two elements"))?;
                    if array.len() != 2 {
                        bail!("Between operator requires exactly two values");
                    }
                    format!("and(gte.{}.{},lte.{}.{})", field, array[0], field, array[1])
                }
                LogicalFilterOperator::Nbetween => {
                    let array = value
                        .as_array()
                        .ok_or(anyhow!("Value must be array with two elements"))?;
                    if array.len() != 2 {
                        bail!("Between operator requires exactly two values");
                    }
                    format!("or(lt.{}.{},gt.{}.{})", field, array[0], field, array[1])
                }
                LogicalFilterOperator::Null => format!("{}.is.null", field),
                LogicalFilterOperator::Nnull => format!("{}.not.is.null", field),
                LogicalFilterOperator::Ina => format!("{}.cs.{}", field, value),
                LogicalFilterOperator::Nina => format!("{}.not.cs.{}", field, value),
            }),
            Filter::ConditionalFilter {
                filters, operator, ..
            } => {
                let conditions: Vec<String> = filters
                    .iter()
                    .map(Self::filter_to_string)
                    .collect::<Result<Vec<_>, _>>()?;

                match operator {
                    ConditionalFilterOperator::And => Ok(format!("and({})", conditions.join(","))),
                    ConditionalFilterOperator::Or => Ok(format!("or({})", conditions.join(","))),
                }
            }
        }
    }
}
*/
