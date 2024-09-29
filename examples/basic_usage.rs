use anyhow::Result;
use ext_crud_rs::clients::supabase::SupabaseClient;
use ext_crud_rs::traits::*;
use ext_crud_rs::{ImplExtendedCrud, PartialStruct};
use serde::{Deserialize, Serialize};
use tokio;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ImplExtendedCrud, PartialStruct)]
#[table_name("users")]
struct User {
    id: Uuid,
    name: String,
    email: String,
    age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplExtendedCrud, PartialStruct)]
#[table_name("products")]
#[partial_struct_name("ProductUpdate")]
struct Product {
    #[primary_key("product_id")]
    product_code: String,
    name: String,
    price: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = SupabaseClient::new("http://localhost:3000");

    // User の例（デフォルトの Partial 名を使用）
    let user = User {
        id: Uuid::new_v4(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
    };

    let _created_user = user.create(&client).await?;

    // Product の例（カスタム Partial 名を使用）
    let product = Product {
        product_code: "PROD-001".to_string(),
        name: "Super Widget".to_string(),
        price: 19.99,
    };

    product.clone().create(&client).await?;
    let mut product_update = product.to_partial();
    product_update.price = Some(24.99);

    let mut new_partial_product = ProductUpdate::new();
    new_partial_product.price = Some(29.99);

    Ok(())
}
