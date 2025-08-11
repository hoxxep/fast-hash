#![doc = include_str!("../README.md")]

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input,
    ConstParam, Data, DeriveInput, Error, Fields, Generics, Index, Lifetime,
    LifetimeParam, Token, TypeParam, WhereClause,
};

fn crate_root() -> TokenStream {
    quote!(::fast_hash)
}

#[proc_macro_derive(FastHash)]
#[allow(non_snake_case)]
pub fn derive_fast_hash(input: TokenStream1) -> TokenStream1 {
    let root = crate_root();
    let hash = quote!(#root::FastHash);
    let hasher_write = quote!(#root::FastHasher);

    let mut input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let mut tokens = TokenStream::new();
    let mut types = Vec::new();

    match input.data {
        // Stability: structs are hashed in the order of their fields.
        Data::Struct(x) => match x.fields {
            Fields::Named(x) => {
                let fields = x.named.iter().map(|x| {
                    types.push(x.ty.clone());
                    x.ident.as_ref().unwrap()
                });
                quote! {
                    #( #hash::hash(&self.#fields, state); )*
                }
                    .to_tokens(&mut tokens)
            }

            Fields::Unnamed(x) => {
                let fields = x.unnamed.iter().enumerate().map(|(i, x)| {
                    types.push(x.ty.clone());
                    Index::from(i)
                });
                quote! {
                    #( #hash::hash(&self.#fields, state); )*
                }
                    .to_tokens(&mut tokens)
            }

            Fields::Unit => (),
        },

        // Stability: (TODO) enums must be keyed with a discriminant for DoS resistance.
        Data::Enum(x) => {
            let mut variant_tokens = TokenStream::new();

            for (discriminant, x) in x.variants.iter().enumerate() {
                let var = &x.ident;

                // TODO(stabilisation): we do this for forward-compatibility, but does it cause other issues?
                // Use write_u8 until discriminant > u8::MAX, then write_u16, u32, u64.
                let span = Span::call_site(); // TODO: improve Span location?
                let (discriminant_method, discriminant_value) = match discriminant {
                    discriminant if discriminant > u32::MAX as usize => {
                        (Ident::new("write_u64", span), Literal::u64_suffixed(discriminant as u64))
                    }
                    discriminant if discriminant > u16::MAX as usize => {
                        (Ident::new("write_u32", span), Literal::u32_suffixed(discriminant as u32))
                    }
                    discriminant if discriminant > u8::MAX as usize => {
                        (Ident::new("write_u16", span), Literal::u16_suffixed(discriminant as u16))
                    }
                    _ => {
                        (Ident::new("write_u8", span), Literal::u8_suffixed(discriminant as u8))
                    }
                };

                match &x.fields {
                    Fields::Named(x) => {
                        let fields: Vec<_> = x
                            .named
                            .iter()
                            .map(|x| {
                                types.push(x.ty.clone());
                                x.ident.as_ref().unwrap()
                            })
                            .collect();
                        // TODO(stabilisation): should we use the enum Name and write_str?
                        //   It would allow re-ordering of named variants without changing
                        //   the hash.
                        quote! {
                            Self::#var { #(#fields),* } => {
                                state.#discriminant_method(#discriminant_value);
                                #( #hash::portable_hash(#fields, state); )*
                            }
                        }
                            .to_tokens(&mut variant_tokens);
                    }

                    Fields::Unnamed(x) => {
                        let fields: Vec<_> = x
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, x)| {
                                types.push(x.ty.clone());
                                format_ident!("_{}", i)
                            })
                            .collect();
                        quote! {
                            Self::#var(#(#fields),*) => {
                                state.#discriminant_method(#discriminant_value);
                                #( #hash::portable_hash(#fields, state); )*
                            }
                        }
                            .to_tokens(&mut variant_tokens);
                    }

                    Fields::Unit => quote! {
                        Self::#var => {
                            state.#discriminant_method(#discriminant_value);
                        },
                    }
                        .to_tokens(&mut variant_tokens),
                }
            }

            // TODO(stability): use a portable discriminant for hashing
            //   named -> use hash(str(variant_name)) + hash(data)
            //   unnamed -> use hash(index) + hash(data)
            //   unit -> use hash(index)
            // Old: #hash::hash(&core::mem::discriminant(self), state);
            quote! {
                match self {
                    #variant_tokens
                }
            }
                .to_tokens(&mut tokens);
        }

        Data::Union(_) => {
            return Error::new(ident.span(), "can't derive `Hash` for union")
                .to_compile_error()
                .into()
        }
    }

    input.generics.make_where_clause();
    let wc = input.generics.where_clause.as_mut().unwrap();
    let where_ = fix_where(Some(wc));
    let SplitGenerics {
        lti,
        ltt,
        tpi,
        tpt,
        cpi,
        cpt,
        wc,
    } = split_generics(&input.generics);
    quote! {
        impl<#(#lti,)* #(#tpi,)* #(#cpi,)*> #hash for #ident<#(#ltt,)* #(#tpt,)* #(#cpt),*> #where_ #wc
            #( #types: #hash ),*
        {
            const HASH_AS_BYTES: bool = {
                let self_size = core::mem::size_of::<Self>();
                let item_size = $(core::mem::size_of::<$types>() + )+ 0;
                let all_as_bytes = $($types::HASH_AS_BYTES && )+ true;
                self_size == item_size && all_as_bytes
            };

            #[inline]
            fn hash<H: #hasher_write>(&self, state: &mut H) {
                if Self::HASH_AS_BYTES {
                    // If we can hash this type as a single write, we do so instead
                    let newlen = core::mem::size_of_val(self);
                    let ptr = self as *const Self as *const u8;
                    // SAFETY: This is safe because we assume that the type is unpadded and does not contain
                    // any pointers or references, so it can be safely treated as a byte slice.
                    state.write_bytes(unsafe { core::slice::from_raw_parts(ptr, newlen) });
                } else {
                    #tokens
                }
            }
        }
    }
        .into()
}

fn fix_where(wc: Option<&mut WhereClause>) -> Option<Token![where]> {
    if let Some(wc) = wc {
        if wc.predicates.is_empty() {
            Some(wc.where_token)
        } else {
            if !wc.predicates.trailing_punct() {
                wc.predicates.push_punct(<Token![,]>::default());
            }
            None
        }
    } else {
        Some(<Token![where]>::default())
    }
}

struct SplitGenerics<
    'a,
    LTI: Iterator<Item = &'a LifetimeParam>,
    LTT: Iterator<Item = &'a Lifetime>,
    TPI: Iterator<Item = &'a TypeParam>,
    TPT: Iterator<Item = &'a Ident>,
    CPI: Iterator<Item = &'a ConstParam>,
    CPT: Iterator<Item = &'a Ident>,
> {
    lti: LTI,
    ltt: LTT,
    tpi: TPI,
    tpt: TPT,
    cpi: CPI,
    cpt: CPT,
    wc: &'a Option<WhereClause>,
}

fn split_generics(
    generics: &Generics,
) -> SplitGenerics<
    '_,
    impl Iterator<Item = &LifetimeParam>,
    impl Iterator<Item = &Lifetime>,
    impl Iterator<Item = &TypeParam>,
    impl Iterator<Item = &Ident>,
    impl Iterator<Item = &ConstParam>,
    impl Iterator<Item = &Ident>,
> {
    SplitGenerics {
        lti: generics.lifetimes(),
        ltt: generics.lifetimes().map(|l| &l.lifetime),
        tpi: generics.type_params(),
        tpt: generics.type_params().map(|t| &t.ident),
        cpi: generics.const_params(),
        cpt: generics.const_params().map(|c| &c.ident),
        wc: &generics.where_clause,
    }
}
