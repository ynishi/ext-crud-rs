use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DeriveInput, Field, Fields,
    Ident, Type,
};

#[proc_macro_derive(ExtendedCrud, attributes(table_name, primary_key))]
pub fn derive_extended_crud(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let table_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table_name"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
        .expect("table_name attribute is required");

    let fields = fields_named_from_input(&input);

    let (primary_key_field, primary_key_type, primary_key_name) = extract_primary_keys(fields);

    let expanded = quote! {
        impl #name {
            fn primary_key_name() -> &'static str {
                #primary_key_name
            }
            fn primary_key(&self) -> &#primary_key_type {
                &self.#primary_key_field
            }
        }
        impl<C: Client> ExtendedCrud<C> for #name {
            type PrimaryKey = #primary_key_type;

            const TABLE_NAME: &'static str = #table_name;

            const PRIMARY_KEY_NAME: &'static str = #primary_key_name;

            fn primary_key(&self) -> &Self::PrimaryKey {
                 &self.#primary_key_field
             }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(PartialEntity, attributes(partial_entity_name, primary_key))]
pub fn partial_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let original_name = &input.ident;

    let partial_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("partial_entity_name"))
        .map(|attr| format_ident!("{}", attr.parse_args::<syn::LitStr>().unwrap().value()))
        .unwrap_or_else(|| format_ident!("Partial{}", original_name.to_string()));

    let fields = fields_named_from_input(&input);

    let (primary_key_field, primary_key_type, primary_key_name) = extract_primary_keys(fields);

    let partial_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>
        }
    });

    let to_partial_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: Some(self.#name.clone())
        }
    });

    let apply_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            if let Some(ref value) = self.#name {
                original.#name = value.clone();
            }
        }
    });

    let new_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

    let expanded = quote! {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct #partial_name {
            #(#partial_fields,)*
        }

        impl #original_name {
            fn to_partial(&self) -> #partial_name {
                #partial_name {
                    #(#to_partial_fields,)*
                }
            }
        }

        impl PartialEntity<#original_name> for #partial_name {
            type PrimaryKey = #primary_key_type;

            const PRIMARY_KEY_NAME: &'static str = #primary_key_name;

            fn new() -> Self {
                 Self {
                     #(#new_fields,)*
                 }
             }

            fn apply_to(&self, original: &#original_name) -> #original_name {
                let mut original = original.clone();
                #(#apply_fields)*
                original
            }

            fn primary_key(&self) -> Option<Self::PrimaryKey> {
                self.#primary_key_field.clone()
            }
        }
    };

    TokenStream::from(expanded)
}

fn fields_named_from_input(input: &DeriveInput) -> &Punctuated<Field, Comma> {
    match &input.data {
        Data::Struct(data) => fields_named(&data.fields),
        _ => panic!("Only structs are supported"),
    }
}

fn fields_named(fields: &Fields) -> &Punctuated<Field, Comma> {
    match fields {
        Fields::Named(fields) => &fields.named,
        _ => panic!("Only named fields are supported"),
    }
}

fn extract_primary_keys(fields: &Punctuated<Field, Comma>) -> (&Ident, &Type, String) {
    fields
        .iter()
        .find(|f| {
            f.attrs
                .iter()
                .any(|attr| attr.path().is_ident("primary_key"))
        })
        .map(|f| {
            let primary_key_field = f.ident.as_ref().unwrap();
            let primary_key_name = f
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("primary_key"))
                .map(|attr| {
                    if let Ok(attr) = attr.parse_args::<syn::LitStr>() {
                        attr.value()
                    } else {
                        primary_key_field.to_string()
                    }
                })
                .unwrap_or(primary_key_field.to_string());
            (primary_key_field, &f.ty, primary_key_name)
        })
        .or_else(|| {
            fields
                .iter()
                .find(|f| f.ident.as_ref().unwrap() == "id")
                .map(|f| (f.ident.as_ref().unwrap(), &f.ty, "id".to_string()))
        })
        .expect("A field named 'id' or with #[primary_key] attribute is required")
}
