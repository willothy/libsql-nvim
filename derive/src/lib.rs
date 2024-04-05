use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromLuaSerde)]
pub fn from_lua(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, _) = generics.split_for_impl();
    let where_clause = match &generics.where_clause {
        Some(where_clause) => quote! { #where_clause, Self: ::serde::Serialize },
        None => quote! { where Self: ::serde::Serialize },
    };

    quote! {
      impl #impl_generics ::mlua::FromLua<'_> for #ident #ty_generics #where_clause {
        #[inline]
        fn from_lua(value: ::mlua::Value<'_>, lua: &'_ ::mlua::Lua) -> ::mlua::Result<Self> {
            lua.from_value(value)
        }
      }
    }
    .into()
}
