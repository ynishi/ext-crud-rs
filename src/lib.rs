pub mod entity;

pub use entity::extend::ExtendedCrud;

pub use entity::extend::PartialEntity;

pub mod clients;

pub use clients::client::Client;

pub mod supabase;

pub use supabase::supabase::SupabaseClient;

#[cfg(feature = "derive")]
pub use ext_crud_derive::*;
