use std::{hint::unreachable_unchecked, ops::Deref};

use quote::{format_ident, quote, spanned::Spanned};
use syn::{parse_macro_input, FnArg, GenericParam, ItemFn, LifetimeDef, Signature, TypeParam, Type, ReturnType, parse_quote_spanned, Pat, parse_quote, Generics};

#[proc_macro_attribute]
pub fn named_fn(
    _attrs: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig:
            Signature {
                constness,
                asyncness,
                unsafety,
                abi,
                fn_token,
                ident,
                generics,
                paren_token,
                inputs,
                variadic,
                output,
            },
        block,
    } = parse_macro_input!(items as ItemFn);

    let mut phantom = Vec::with_capacity(generics.params.len());
    let mut phantom_new = Vec::with_capacity(generics.params.len());

    let args = try_collect::<_, _, _, Vec<_>>(inputs.iter().map(|x| match x {
        FnArg::Typed(x) => Ok((&x.ty as &Type, &x.pat as &Pat)),
        FnArg::Receiver(x) => Err(syn::Error::new_spanned(x, "Expected typed argument")),
    }));
    let (arg_tys, arg_pats) = match args {
        Ok(x) => x.into_iter().unzip::<_, _, Vec<_>, Vec<_>>(),
        Err(e) => return e.to_compile_error().into()
    };

    let output = match output {
        x @ ReturnType::Default => parse_quote_spanned! { x.__span() => () },
        ReturnType::Type(_, ty) => ty
    };

    for param in generics.params.iter() {
        match param {
            GenericParam::Type(TypeParam { attrs, ident, .. }) => {
                phantom.push(quote! { #(#attrs)* named_fn_core::marker::PhantomData<#ident> });
                phantom_new.push(quote! { named_fn_core::marker::PhantomData });
            },
            _ => continue,
        };
    }

    let mut struct_generics = Generics::default();
    struct_generics.params = generics.params.iter().filter_map(|x| match x {
        GenericParam::Lifetime(_) => None,
        other => Some(other)
    }).cloned().collect();

    let (_, struct_ty_gen, _) = struct_generics.split_for_impl();
    let (impl_gen, _, where_gen) = generics.split_for_impl();
    let mut struct_ident = format_ident!("{}", to_pascal_case(&ident.to_string()));
    struct_ident.set_span(ident.span());

    return quote! {
        extern crate core as named_fn_core;
        
        #(#attrs)*
        #vis struct #struct_ident #struct_ty_gen (#(#phantom),*);

        impl #struct_ty_gen #struct_ident #struct_ty_gen {
            #[inline]
            #vis const fn new () -> Self {
                return Self (#(#phantom_new),*);
            }
        }

        impl #impl_gen named_fn_core::ops::FnOnce<(#(#arg_tys,)*)> for #struct_ident #struct_ty_gen #where_gen {
            type Output = #output;
            extern "rust-call" fn call_once(self, (#(#arg_pats,)*): (#(#arg_tys,)*)) -> Self::Output #block
        }

        impl #impl_gen named_fn_core::ops::FnMut<(#(#arg_tys,)*)> for #struct_ident #struct_ty_gen #where_gen {
            extern "rust-call" fn call_mut(&mut self, (#(#arg_pats,)*): (#(#arg_tys,)*)) -> Self::Output #block
        }

        impl #impl_gen named_fn_core::ops::Fn<(#(#arg_tys,)*)> for #struct_ident #struct_ty_gen #where_gen {
            extern "rust-call" fn call(&self, (#(#arg_pats,)*): (#(#arg_tys,)*)) -> Self::Output #block
        }

        impl #struct_ty_gen named_fn_core::default::Default for #struct_ident #struct_ty_gen {
            #[inline]
            fn default () -> Self {
                Self::new()
            }
        }
    }
    .into();
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut is_upper = true;

    for c in s.chars() {
        if c == '_' {
            is_upper = true;
            continue;
        }

        if is_upper {
            result.extend(c.to_uppercase());
            is_upper = false;
        } else {
            result.extend(c.to_lowercase())
        }
    }

    return result;
}

#[inline]
fn try_collect<T, E, I: Iterator<Item = Result<T, E>>, C: Default + FromIterator<T>>(
    mut iter: I,
) -> Result<C, E> {
    let collect = (&mut iter)
        .take_while(Result::is_ok)
        .map(|x| unsafe { x.unwrap_unchecked() })
        .collect::<C>();

    return match iter.next() {
        Some(Err(e)) => Err(e),
        None => Ok(collect),
        _ => unsafe { unreachable_unchecked() },
    };
}
