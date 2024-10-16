pub mod entity;

pub use entity::extend::ExtendedCrud;

pub use entity::extend::PartialEntity;

pub use entity::extend::TryFromError;

pub mod clients;

pub use clients::client::Client;

pub mod supabase;

pub use supabase::supabase::SupabaseClient;

#[cfg(feature = "derive")]
pub use ext_crud_derive::*;

/// Re-export the derive macros and the traits.
/// Easy to use in the client code, just import this module with
/// `use ext_crud_rs::prelude::*;`
pub mod prelude {
    pub use crate::clients::client::Client;
    pub use crate::entity::extend::ExtendedCrud;
    pub use crate::entity::extend::PartialEntity;
    pub use crate::entity::extend::TryFromError;
    pub use crate::supabase::supabase::SupabaseClient;

    #[cfg(feature = "derive")]
    pub use ext_crud_derive::*;
}
