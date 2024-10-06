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
    #[serde(rename = "product_id")] // TODO support only use primary_key(auto rename)
    product_code: String,
    name: String,
    price: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Supabase Local Environment の URL を使用
    let supabase_api_url = "http://127.0.0.1:54321";
    let supabase_service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").map_err(|_| {
        anyhow::anyhow!("Please set the SUPABASE_SERVICE_ROLE_KEY environment variable")
    })?;
    let client = SupabaseClient::new(supabase_api_url, &supabase_service_role_key);

    // User の例（デフォルトの Partial 名を使用）
    let mut user = User {
        id: Uuid::new_v4(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
    };

    user.clone().create(&client).await?;
    let crated_user = User::read(&client, user.id).await?;
    println!("Crated User: {:?}", crated_user);

    user.age = 40;
    user.update(&client).await?;

    let updated_user = User::read(&client, user.id).await?;
    println!("Updated User: {:?}", updated_user);

    // Product の例（カスタム Partial 名を使用）
    let mut product = Product {
        product_code: Uuid::new_v4().to_string(), //"PROD-001".to_string(),
        name: "Super Widget".to_string(),
        price: 19.99,
    };

    product.clone().create(&client).await?;
    let created_product = Product::read(&client, product.product_code.clone()).await?;
    println!("Created Product: {:?}", created_product);

    product.price = 24.99;
    product.update(&client).await?;
    let updated_product = Product::read(&client, product.product_code.clone()).await?;
    println!("Updated Product: {:?}", updated_product);

    let mut product_update = product.to_partial();
    product_update.price = Some(24.99);

    let mut new_partial_product = ProductUpdate::new();
    new_partial_product.price = Some(29.99);

    Ok(())
}
