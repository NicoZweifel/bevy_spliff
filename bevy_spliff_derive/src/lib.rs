use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index, Type, parse_macro_input};

const ONLY_STRUCTS: &str = "Joinable can only be derived for structs";
const NO_FIELDS: &str = "Joinable struct must have at least one field";
const AMBIGUOUS_FIELDS: &str = "Multiple fields found. Please mark the join target with #[join]";

#[proc_macro_derive(Joinable, attributes(join, relationship, relationship_target))]
pub fn derive_joinable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_joinable(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand_joinable(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

    Ok(quote! {
        impl #impl_generics Joinable for #name #ty_generics #where_clause {
            type Mapper = #mapper;
            type Out<'a> = #out where Self: 'a;

            fn targets(&self) -> Self::Out<'_> {
                #target_expr
            }
        }
    })
}

fn is_vec(ty: &Type) -> bool {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        seg.ident == "Vec"
    } else {
        false
    }
}

fn analyze_fields(fields: &syn::Fields) -> syn::Result<(usize, &syn::Field)> {
    fields
        .iter()
        .enumerate()
        .find(|(_, f)| {
            f.attrs.iter().any(|attr| {
                attr.path().is_ident("join")
                    || attr.path().is_ident("relationship")
                    || attr.path().is_ident("relationship_target")
            })
        })
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
    use syn::parse_quote;

    #[test]
    fn single_tuple_should_fallback_to_single_join() {
        let input: DeriveInput = parse_quote! { struct Player(Entity); };

        let res = expand_joinable(input);

        let output = res.unwrap().to_string();
        assert!(output.contains("SingleJoin"));
        assert!(output.contains("self . 0"));
    }

    #[test]
    fn single_tuple_vec_should_fallback_to_multiple_join() {
        let input: DeriveInput = parse_quote! { struct Player(Vec<Entity>); };

        let res = expand_joinable(input);

        let output = res.unwrap().to_string();
        assert!(output.contains("MultipleJoin"));
        assert!(output.contains("self . 0"));
    }

    #[test]
    fn multi_tuple_with_join_should_target_index() {
        let input: DeriveInput = parse_quote! { struct Offset(f32, #[join] Entity); };

        let res = expand_joinable(input);

        let output = res.unwrap().to_string();
        assert!(output.contains("self . 1"));
    }

    #[test]
    fn named_field_with_join_should_target_ident() {
        let input: DeriveInput = parse_quote! { struct Armor { val: f32, #[join] target: Entity } };

        let res = expand_joinable(input);

        let output = res.unwrap().to_string();
        assert!(output.contains("self . target"));
    }

    #[test]
    fn relationship_attribute_should_target_field() {
        let input: DeriveInput = parse_quote! {
            struct WeaponOf(#[relationship(target = Weapons)] Entity);
        };

        let res = expand_joinable(input);

        let output = res.unwrap().to_string();
        assert!(output.contains("self . 0"));
    }

    #[test]
    fn relationship_target_attribute_should_target_vec_field() {
        let input: DeriveInput = parse_quote! {
            struct Weapons(#[relationship_target(rel = WeaponOf)] Vec<Entity>);
        };

        let res = expand_joinable(input);

        let output = res.unwrap().to_string();
        assert!(output.contains("MultipleJoin"));
        assert!(output.contains("self . 0"));
    }

    #[test]
    fn is_vec_should_fail_on_non_vec_path() {
        let ty: Type = parse_quote! { Option<Entity> };

        let res = is_vec(&ty);

        assert!(!res);
    }

    #[test]
    fn is_vec_should_fail_on_non_path() {
        let ty: Type = parse_quote! { &Entity };

        let res = is_vec(&ty);

        assert!(!res);
    }

    #[test]
    fn is_vec_should_fail_on_plain_type() {
        let ty: Type = parse_quote! { u32 };

        let res = is_vec(&ty);

        assert!(!res);
    }

    #[test]
    fn expansion_should_error_on_enum() {
        let input: DeriveInput = parse_quote! { enum NotStruct { V } };

        let res = expand_joinable(input);

        assert_eq!(res.unwrap_err().to_string(), ONLY_STRUCTS);
    }

    #[test]
    fn expansion_should_error_on_union() {
        let input: DeriveInput = parse_quote! { union NotSupported { f: Entity } };

        let res = expand_joinable(input);

        assert_eq!(res.unwrap_err().to_string(), ONLY_STRUCTS);
    }

    #[test]
    fn expansion_should_error_on_empty_struct() {
        let input: DeriveInput = parse_quote! { struct Empty; };

        let res = expand_joinable(input);

        assert_eq!(res.unwrap_err().to_string(), NO_FIELDS);
    }

    #[test]
    fn expansion_should_error_on_ambiguous_fields() {
        let input: DeriveInput = parse_quote! { struct Ambi(Entity, Entity); };

        let res = expand_joinable(input);

        assert_eq!(res.unwrap_err().to_string(), AMBIGUOUS_FIELDS);
    }
}
