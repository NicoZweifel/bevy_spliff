use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index, Type, parse_macro_input};

const ONLY_STRUCTS: &str = "Joinable can only be derived for structs";
const NO_FIELDS: &str = "Joinable struct must have at least one field";
const AMBIGUOUS_FIELDS: &str = "Multiple fields found. Please mark the join target with #[join]";

#[proc_macro_derive(Joinable, attributes(join))]
pub fn derive_joinable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_joinable(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn expand_joinable(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => return Err(syn::Error::new_spanned(&input.ident, ONLY_STRUCTS)),
    };

    let (index, field) = analyze_fields(fields)?;
    let access_expr = if let Some(ident) = &field.ident {
        quote! { self.#ident }
    } else {
        let syn_index = Index::from(index);
        quote! { self.#syn_index }
    };

    let (mapper, out, target_expr) = if is_vec(&field.ty) {
        (
            quote! { MultipleJoin },
            quote! { std::iter::Cloned<std::slice::Iter<'a, bevy_ecs::prelude::Entity>> },
            quote! { #access_expr.iter().cloned() },
        )
    } else {
        (
            quote! { SingleJoin },
            quote! { std::iter::Once<bevy_ecs::prelude::Entity> },
            quote! { std::iter::once(#access_expr) },
        )
    };

    let expanded = quote! {
        impl #impl_generics Joinable for #name #ty_generics #where_clause {
            type Mapper = #mapper;
            type Out<'a> = #out where Self: 'a;

            fn targets(&self) -> Self::Out<'_> {
                #target_expr
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

fn is_vec(ty: &Type) -> bool {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        return seg.ident == "Vec";
    }
    false
}

fn analyze_fields(fields: &syn::Fields) -> syn::Result<(usize, &syn::Field)> {
    fields
        .iter()
        .enumerate()
        .find(|(_, f)| f.attrs.iter().any(|attr| attr.path().is_ident("join")))
        .or_else(|| {
            (fields.len() == 1)
                .then(|| fields.iter().enumerate().next())
                .flatten()
        })
        .ok_or_else(|| {
            if fields.is_empty() {
                syn::Error::new_spanned(fields, NO_FIELDS)
            } else {
                syn::Error::new_spanned(fields, AMBIGUOUS_FIELDS)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{ItemStruct, parse_quote};

    #[test]
    fn single_tuple_should_fallback() {
        let st: ItemStruct = parse_quote! { struct Player(Entity); };
        let (idx, field) = analyze_fields(&st.fields).unwrap();
        assert_eq!(idx, 0);
        assert!(!is_vec(&field.ty));
    }

    #[test]
    fn single_tuple_vec_should_fallback() {
        let st: ItemStruct = parse_quote! { struct Player(Vec<Entity>); };
        let (idx, field) = analyze_fields(&st.fields).unwrap();
        assert_eq!(idx, 0);
        assert!(is_vec(&field.ty));
    }

    #[test]
    fn multi_tuple_should_offset() {
        let st: ItemStruct = syn::parse_quote! {
            struct Offset(f32, #[join] Entity);
        };
        let (idx, _) = analyze_fields(&st.fields).unwrap();

        assert_eq!(idx, 1);
    }

    #[test]
    fn named_field_attribute_should_target() {
        let st: ItemStruct = parse_quote! {
            struct Armor {
                val: f32,
                #[join]
                target: Entity
            }
        };

        let (idx, field) = analyze_fields(&st.fields).unwrap();

        assert_eq!(idx, 1);
        assert_eq!(field.ident.as_ref().unwrap(), "target");
        assert!(field.attrs.iter().any(|a| a.path().is_ident("join")));
    }

    #[test]
    fn ambiguous_multi_field_should_error() {
        let st: ItemStruct = parse_quote! {
            struct Ambiguous { a: Entity, b: Entity }
        };

        let result = analyze_fields(&st.fields);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), AMBIGUOUS_FIELDS);
    }

    #[test]
    fn empty_struct_should_error() {
        let st: ItemStruct = parse_quote! { struct Empty; };

        let result = analyze_fields(&st.fields);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), NO_FIELDS);
    }

    #[test]

    fn empty_braces_should_error() {
        let st: ItemStruct = parse_quote! { struct Empty {} };

        let result = analyze_fields(&st.fields);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), NO_FIELDS);
    }

    #[test]
    fn not_struct_should_error() {
        let input: DeriveInput = parse_quote! {
            enum NotStruct { Variant }
        };

        let result = expand_joinable(input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), ONLY_STRUCTS);
    }
}
