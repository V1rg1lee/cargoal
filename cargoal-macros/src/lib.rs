use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Meta};

/// Macro to derive the `Entity` trait for a struct
/// The `Entity` trait provides information about the struct's table name, columns, primary keys and types
///
/// Args:
/// input: The struct to derive the `Entity` trait for
///
/// Returns:
/// The struct with the `Entity` trait implemented
///
/// # Example
/// ```rust
/// extern crate cargoal_macros;
/// use cargoal_macros::Entity;
///
/// #[derive(Entity)]
/// struct User {
///     #[column("user_id")]
///     #[primary_key]
///     id: i32,
///
///     #[column("username")]
///     username: String,
///
///     #[column("email")]
///     #[unique]
///     email: String,
/// }
///
/// fn main() {
///     assert_eq!(User::TABLE_NAME, "user");
///     assert_eq!(User::COLUMNS, &["id", "username", "email"]);
///     assert_eq!(User::TYPES, &["i32", "String", "String"]);
///     assert_eq!(User::primary_keys(), vec!["id"]);
/// }
/// ```
#[proc_macro_derive(Entity, attributes(table, column, primary_key, unique, default))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let table_name = parse_table_name(&input);
    let (columns, primary_keys, types) = parse_fields(&input);

    generate_impl(struct_name, &table_name, &columns, &primary_keys, &types)
}

/// Extract the table name from the struct's attributes
/// If no table attribute is found, the table name is the struct's name in lowercase
///
/// ## Args:
/// - input: The struct to extract the table name from
///
/// ## Returns:
/// - The table name
fn parse_table_name(input: &DeriveInput) -> String {
    let mut table_name = input.ident.to_string().to_lowercase();

    for attr in &input.attrs {
        if let Ok(Meta::NameValue(nv)) = attr.meta.clone().try_into() {
            if attr.path().is_ident("table") {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let syn::Lit::Str(lit) = &expr_lit.lit {
                        table_name = lit.value();
                    }
                }
            }
        }
    }

    table_name
}

/// Extract the columns, primary keys and types from the struct's fields
///
/// ## Args:
/// - input: The struct to extract the fields from
///
/// ## Returns:
/// - A tuple containing the columns, primary keys and types
fn parse_fields(input: &DeriveInput) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut columns = Vec::new();
    let mut primary_keys = Vec::new();
    let mut types = Vec::new();

    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => panic!("#[derive(Entity)] ne peut être appliqué qu'à des structs"),
    };

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let mut column_name = field_name.to_string();
        let mut is_primary_key = false;

        for attr in &field.attrs {
            if let Ok(Meta::NameValue(nv)) = attr.meta.clone().try_into() {
                if attr.path().is_ident("column") {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(lit) = &expr_lit.lit {
                            column_name = lit.value();
                        }
                    }
                }
            } else if attr.path().is_ident("primary_key") {
                is_primary_key = true;
            }
        }

        columns.push(column_name.clone());
        types.push(format!("{}", quote! { #field_type }));

        if is_primary_key {
            primary_keys.push(column_name);
        }
    }

    (columns, primary_keys, types)
}

/// Generate the implementation of the `Entity` trait for the struct
///
/// ## Args:
/// - struct_name: The name of the struct
/// - table_name: The name of the table
/// - columns: The columns of the table
/// - primary_keys: The primary keys of the table
/// - types: The types of the columns
///
/// ## Returns:
/// - The generated implementation
fn generate_impl(
    struct_name: &syn::Ident,
    table_name: &str,
    columns: &[String],
    primary_keys: &[String],
    types: &[String],
) -> TokenStream {
    let table_name_lit = table_name.to_string();
    let columns_lit = columns.iter().map(|col| col.as_str());
    let primary_keys_lit = primary_keys.iter().map(|col| col.as_str());
    let types_lit = types.iter().map(|t| t.as_str());

    let generated = quote! {
        impl #struct_name {
            pub const TABLE_NAME: &'static str = #table_name_lit;
            pub const COLUMNS: &'static [&'static str] = &[#(#columns_lit),*];
            pub const TYPES: &'static [&'static str] = &[#(#types_lit),*];

            pub fn primary_keys() -> Vec<&'static str> {
                vec![#(#primary_keys_lit),*]
            }
        }
    };

    TokenStream::from(generated)
}
