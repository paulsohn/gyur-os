//! A crate which defines [`StructuralOf`] proc-macro.

use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, parse_str, Index};

/// Use this derivation to field structs so that a volatile pointer to it has field-wise access.
#[proc_macro_derive(StructuralOf)]
pub fn derive_bounded_structural_of(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        vis,
        ident: orig_ident,
        data,
        ..
    } = syn::parse(input).unwrap();

    let structural_ident = parse_str::<Ident>(
        format!("StructuralOf{}", orig_ident).as_str()
    ).unwrap();

    let fields = match data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("`StructuralOf` can be derived only for non-unit structs."),
    };
    if let syn::Fields::Unit = fields {
        panic!("`StructuralOf` can be derived only for non-unit structs.");
    }

    let field_methods = fields.iter().enumerate()
        .map(|(i, field)| {
            let vis = field.vis.clone();
            let ty = field.ty.clone();

            match field.ident.as_ref() {
                Some(id) => {
                    let ident = id.clone();
                    quote! {
                        #vis fn #ident(self) -> volatile::VolatilePtr<'a, #ty, A> {
                            let ptr = self.0;
                            volatile::map_field!(ptr.#ident)
                        }
                    }
                },
                None => {
                    let method = parse_str::<Ident>(format!("field_{}", i).as_str()).unwrap();
                    let idx = Index::from(i);
                    quote! {
                        #vis fn #method(self) -> volatile::VolatilePtr<'a, #ty, A> {
                            let ptr = self.0;
                            volatile::map_field!(ptr.#idx)
                        }
                    }
                },
            }
        });
    
    let tokens = quote! {
        #[allow(missing_docs)]
        #[allow(missing_debug_implementations)]
        #vis struct #structural_ident<'a, A: volatile::access::Access>(
            volatile::VolatilePtr<'a, #orig_ident, A>
        );
        impl<'a, A: volatile::access::Access> #structural_ident<'a, A> {
            #(
                #[allow(missing_docs)]
                #field_methods
            )*
        }
        impl<'a, A: volatile::access::Access> volatile_field::Structural<'a, #orig_ident, A> for volatile::VolatilePtr<'a, #orig_ident, A> {
            type StructuralType = #structural_ident<'a, A>;

            fn fields(self) -> Self::StructuralType {
                #structural_ident(self)
            }
        }
    };
    tokens.into()
}