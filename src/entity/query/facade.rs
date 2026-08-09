use super::core::{QueryContext, QueryField};

/// The trait that defines the query facade.
pub trait QueryFacade
where
    // Convert Request -> Context
    QueryContext<Self::Field>: TryFrom<Self::Request, Error = Self::Error>,
{
    type Field: QueryField;
    type Request;
    type Error;
}
