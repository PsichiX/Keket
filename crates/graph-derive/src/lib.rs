use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, ItemStruct, parse_macro_input, parse_quote};

fn has_asset_deps_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("asset_deps"))
}

/// Derives the `AssetTree` trait for a struct.
/// This macro will automatically implement the `AssetTree` trait for the struct,
/// allowing it to return its asset dependencies based on the fields that have
/// the `#[asset_deps]` attribute.
#[proc_macro_derive(AssetTree, attributes(asset_deps))]
pub fn asset_tree_struct(input: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        mut generics,
        ..
    } = parse_macro_input!(input as ItemStruct);
    let generics_params = generics.params.clone();
    let where_clause = generics.make_where_clause();
    for param in &generics_params {
        if let syn::GenericParam::Type(ty) = param {
            where_clause.predicates.push(parse_quote!(#ty: AssetTree));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields = fields
        .iter()
        .filter(|field| has_asset_deps_attr(&field.attrs))
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<_>>();
    quote! {
        impl #impl_generics AssetTree for #ident #ty_generics #where_clause {
            fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
                let mut result = vec![];
                #(
                    result.extend(self.#fields.asset_dependencies());
                )*
                result
            }
        }
    }
    .into()
}
