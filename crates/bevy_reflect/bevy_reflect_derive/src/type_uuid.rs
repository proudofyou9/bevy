use bevy_macro_utils::BevyManifest;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{DeriveInput, Expr, ExprLit, Generics, Ident, Lit, LitInt, LitStr, Meta};
use uuid::Uuid;

pub(crate) fn type_uuid_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut uuid = None;

    #[allow(clippy::manual_let_else)]
    for attribute in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("uuid"))
    {
        let Meta::NameValue(ref name_value) = attribute.meta else {
            continue;
        };

        let uuid_str = match &name_value.value {
            Expr::Lit(ExprLit{lit: Lit::Str(lit_str), ..}) => lit_str,
            _ => return Err(syn::Error::new_spanned(attribute, "`uuid` attribute must take the form `#[uuid = \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"]`.")),
        };

        uuid =
            Some(Uuid::parse_str(&uuid_str.value()).map_err(|err| {
                syn::Error::new_spanned(uuid_str, format!("Invalid UUID: {err}"))
            })?);
    }

    let uuid = uuid.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "No `#[uuid = \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"]` attribute found.",
        )
    })?;

    Ok(gen_impl_type_uuid(TypeUuidDef {
        type_ident: input.ident,
        generics: input.generics,
        uuid,
    }))
}

/// Generates an implementation of `TypeUuid`. If there any generics, the `TYPE_UUID` will be a composite of the generic types' `TYPE_UUID`.
pub(crate) fn gen_impl_type_uuid(def: TypeUuidDef) -> TokenStream {
    let uuid = def.uuid;
    let mut generics = def.generics;
    let ty = def.type_ident;

    let bevy_reflect_path = BevyManifest::default().get_path("bevy_reflect");

    generics.type_params_mut().for_each(|param| {
        param
            .bounds
            .push(syn::parse_quote!(#bevy_reflect_path::TypeUuid));
    });

    let bytes = uuid
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:#X}"))
        .map(|byte_str| syn::parse_str::<LitInt>(&byte_str).unwrap());

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let base = quote! { #bevy_reflect_path::Uuid::from_bytes([#( #bytes ),*]) };
    let type_uuid = generics.type_params().enumerate().fold(base, |acc, (index, param)| {
        let ident = &param.ident;
        let param_uuid = quote!(
            #bevy_reflect_path::Uuid::from_u128(<#ident as #bevy_reflect_path::TypeUuid>::TYPE_UUID.as_u128().wrapping_add(#index as u128))
        );
        quote! {
            #bevy_reflect_path::__macro_exports::generate_composite_uuid(#acc, #param_uuid)
        }
    });

    quote! {
        impl #impl_generics #bevy_reflect_path::TypeUuid for #ty #type_generics #where_clause {
            const TYPE_UUID: #bevy_reflect_path::Uuid = #type_uuid;
        }
    }
}

/// A struct containing the data required to generate an implementation of `TypeUuid`. This can be generated by either [`impl_type_uuid!`][crate::impl_type_uuid!] or [`type_uuid_derive`].
pub(crate) struct TypeUuidDef {
    pub type_ident: Ident,
    pub generics: Generics,
    pub uuid: Uuid,
}

impl Parse for TypeUuidDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let type_ident = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;
        input.parse::<Comma>()?;
        let uuid = input.parse::<LitStr>()?.value();
        let uuid = Uuid::parse_str(&uuid).map_err(|err| input.error(format!("{err}")))?;

        Ok(Self {
            type_ident,
            generics,
            uuid,
        })
    }
}
