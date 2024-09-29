use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ImplExtendedCrud, attributes(table_name, primary_key))]
pub fn derive_extended_crud(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let table_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table_name"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
        .expect("table_name attribute is required");

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let (primary_key_field, primary_key_type) = fields
        .iter()
        .find(|f| {
            f.attrs
                .iter()
                .any(|attr| attr.path().is_ident("primary_key"))
        })
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .or_else(|| {
            fields
                .iter()
                .find(|f| f.ident.as_ref().unwrap() == "id")
                .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        })
        .expect("A field named 'id' or with #[primary_key] attribute is required");

    let expanded = quote! {
        impl<C: Client> ExtendedCrud<C> for #name {
            type PrimaryKey = #primary_key_type;

            const TABLE_NAME: &'static str = #table_name;

            fn primary_key(&self) -> &Self::PrimaryKey {
                 &self.#primary_key_field
             }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(PartialStruct, attributes(partial_struct_name))]
pub fn patial_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let partial_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("partial_struct_name"))
        .map(|attr| format_ident!("{}", attr.parse_args::<syn::LitStr>().unwrap().value()))
        .unwrap_or_else(|| format_ident!("Partial{}", name.to_string()));

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

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

        impl #name {
            pub fn to_partial(&self) -> #partial_name {
                #partial_name {
                    #(#to_partial_fields,)*
                }
            }
        }

        impl #partial_name {
            pub fn new() -> Self {
                 Self {
                     #(#new_fields,)*
                 }
             }

            pub fn apply_to(&self, original: &mut #name) {
                #(#apply_fields)*
            }
        }
    };

    TokenStream::from(expanded)
}
