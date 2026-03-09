use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

#[proc_macro_derive(Joinable)]
pub fn derive_joinable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => panic!("Joinable can only be derived for structs"),
    };

    let (field_type, access_expr) = match fields {
        Fields::Unnamed(f) => {
            let ty = &f
                .unnamed
                .first()
                .expect("Joinable struct must have a field")
                .ty;
            (ty, quote! { self.0 })
        }
        Fields::Named(f) => {
            let field = f.named.first().expect("Joinable struct must have a field");
            let ident = &field.ident;
            (&field.ty, quote! { self.#ident })
        }
        _ => panic!("Unit structs cannot be Joinable"),
    };

    let is_multiple = is_vec(field_type);

    let (mapper, out_type, target_expr) = if is_multiple {
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
            type Out<'a> = #out_type where Self: 'a;

            fn targets(&self) -> Self::Out<'_> {
                #target_expr
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_vec(ty: &Type) -> bool {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        return seg.ident == "Vec";
    }
    false
}
