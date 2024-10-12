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

    let mut partial_product = product.to_partial();
    partial_product.price = Some(26.99);
    println!("Partial Product: {:?}", partial_product);
    let applied_product = partial_product.apply_to(&updated_product);
    println!("Applied Product: {:?}", applied_product);
    applied_product.update(&client).await?;
    let updated_applied_product = Product::read(&client, product.product_code.clone()).await?;
    println!("Updated Partial Product: {:?}", updated_applied_product);

    let mut new_partial_product = ProductUpdate::new();
    new_partial_product.price = Some(29.99);
    println!("New Partial Product: {:?}", new_partial_product);

    Ok(())
}

#[test]
fn test_partial_no_specific_name() {
    let id = Uuid::new_v4();
    let user = User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
    };
    assert_eq!(user.primary_key(), &id);
    assert_eq!(User::primary_key_name(), "id");

    let mut partial_user = user.to_partial();
    assert_eq!(partial_user.id, Some(id));
    assert_eq!(partial_user.name, Some("John Doe".to_string()));
    assert_eq!(partial_user.email, Some("john@example.com".to_string()));
    assert_eq!(partial_user.age, Some(30));
    partial_user.age = Some(40);
    assert_eq!(partial_user.primary_key(), Some(id));
    let updated = partial_user.apply_to(&user);
    assert_eq!(40, updated.age);
    assert_eq!(user.id, updated.id);
}

#[test]
fn test_partial_specific_name() {
    let code_or_id = "PROD-001";
    let product = Product {
        product_code: code_or_id.to_string(),
        name: "John Doe Product".to_string(),
        price: 30.0,
    };
    assert_eq!(*product.primary_key(), code_or_id.to_string());

    let mut partial_product = product.to_partial();
    assert_eq!(partial_product.product_code, Some(code_or_id.to_string()));
    assert_eq!(partial_product.name, Some("John Doe Product".to_string()));
    assert_eq!(partial_product.price, Some(30.0));
    assert_eq!(partial_product.primary_key(), Some(code_or_id.to_string()));
    partial_product.price = Some(40.0);
    let updated = partial_product.apply_to(&product);
    assert_eq!(40.0, updated.price);
}
