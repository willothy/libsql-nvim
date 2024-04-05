use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, ImplItemFn};

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

struct ReplaceSelfVisitor {
    replacement_ident: syn::Ident,
}

impl syn::visit_mut::VisitMut for ReplaceSelfVisitor {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if let syn::Expr::Path(expr_path) = expr {
            if let Some(segment) = expr_path.path.segments.first_mut() {
                if segment.ident == "self" {
                    segment.ident = self.replacement_ident.clone();
                }
            }
        }
        syn::visit_mut::visit_expr_mut(self, expr);
    }
}

#[proc_macro_attribute]
pub fn luv_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ImplItemFn {
        vis,
        mut sig,
        mut block,
        ..
    } = parse_macro_input!(item as syn::ImplItemFn);

    let return_type = &sig.output;
    let output = match return_type.clone() {
        syn::ReturnType::Type(_, t) => t,
        syn::ReturnType::Default => panic!(),
    };
    sig.output = parse_quote! { -> ::mlua::Result<()> };

    sig.asyncness = None;

    let self_clone = if sig.receiver().is_some() {
        let replacement_ident = syn::Ident::new("__self", Span::call_site());

        syn::visit_mut::visit_block_mut(&mut ReplaceSelfVisitor { replacement_ident }, &mut block);
        quote! { let __self = self.clone(); }
    } else {
        quote! {}
    };

    quote! {
        #vis #sig {
            let data = ::std::sync::Arc::new(::tokio::sync::Mutex::new(None));

            let handle = ::nvim_oxi::libuv::AsyncHandle::new({
                let data = ::std::sync::Arc::clone(&data);
                move || {
                    let Some(rv) = data.blocking_lock().take() else {
                        return ::mlua::Result::Err(mlua::Error::RuntimeError(
                            "data not set".to_string(),
                        ));
                    };
                    cb.call(rv)
                }
            })
            .into_lua_err()?;

            ::std::thread::spawn({
                #self_clone
                move || {
                    let rt = ::tokio::runtime::Runtime::new().into_lua_err()?;
                    rt.block_on(async {
                        let res: #output = async { #block }.await;

                        data.lock()
                            .await
                            .replace(res);
                        handle.send().into_lua_err()?;
                        ::mlua::Result::Ok(())
                    })
                }
            });

            Ok(())
        }
    }
    .into()
}
