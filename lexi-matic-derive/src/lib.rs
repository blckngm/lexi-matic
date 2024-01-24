extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use regex_automata::{
    dfa::{dense::DFA, StartKind},
    MatchKind,
};
use syn::{parse_macro_input, Data, DeriveInput, LitStr};

/// Derive the Lexer implementation.
#[proc_macro_derive(Lexer, attributes(regex, token))]
pub fn derive_lexer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    derive_lexer_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_lexer_impl(item: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let e = match item.data {
        Data::Enum(e) => e,
        _ => return Err(syn::Error::new_spanned(item, "expect an enum")),
    };
    let name = item.ident;
    let mut regexes = Vec::with_capacity(e.variants.len());
    let mut matches = Vec::new();
    for (i, v) in e.variants.iter().enumerate() {
        let vn = &v.ident;
        let i = i as u32;
        matches.push(if v.fields.is_empty() {
            quote! {
                #i => #name::#vn,
            }
        } else {
            quote! {
                #i => #name::#vn(&remaining[..len]),
            }
        });

        let mut regex = None;
        for a in &v.attrs {
            let r = if a.path().is_ident("regex") {
                let v = &a.meta.require_name_value()?.value;
                let x: LitStr = syn::parse_quote!(#v);
                x.value()
            } else if a.path().is_ident("token") {
                let v = &a.meta.require_name_value()?.value;
                let x: LitStr = syn::parse_quote!(#v);
                regex_syntax::escape(&x.value())
            } else {
                continue;
            };
            if regex.is_none() {
                regex = Some(r);
            } else if regex.is_some() {
                return Err(syn::Error::new_spanned(
                    a,
                    "duplicated regex or token atrribute",
                ));
            }
        }
        match regex {
            None => {
                return Err(syn::Error::new_spanned(
                    v,
                    "missing a regex or token attribute",
                ))
            }
            Some(r) => regexes.push(r),
        }
    }

    let dfa = DFA::builder()
        .configure(
            DFA::config()
                // Use MatchKind::All to get longest match.
                .match_kind(MatchKind::All)
                .start_kind(StartKind::Anchored)
                .minimize(true),
        )
        .build_many(&regexes)
        .unwrap();
    let (little_bytes, little_p) = dfa.to_bytes_little_endian();
    let (big_bytes, big_p) = dfa.to_bytes_big_endian();
    let little_bytes = &little_bytes[little_p..];
    let big_bytes = &big_bytes[big_p..];
    let ll = little_bytes.len();
    let bl = big_bytes.len();
    let dfa = quote! {
        #[repr(C, align(4))]
        struct Align4<T>(T);
        #[cfg(target_endian = "little")]
        static __DFA_BYTES: &Align4<[u8; #ll]> = &Align4([ #(#little_bytes),* ]);
        #[cfg(target_endian = "big")]
        static __DFA_BYTES: &Align4<[u8; #bl]> = &Align4([ #(#big_bytes),* ]);
        static DFA: std::sync::OnceLock<regex_automata::dfa::dense::DFA<&[u32]>> = std::sync::OnceLock::new();
        let dfa = DFA.get_or_init(||
            regex_automata::dfa::dense::DFA::from_bytes(&__DFA_BYTES.0).unwrap().0
        );
    };

    let gen = if item.generics.lt_token.is_some() {
        quote!(<'a>)
    } else {
        quote!()
    };
    let iter_name = format_ident!("{name}Iterator");
    let lexer_impl = quote! {
        impl <'a> lexi_matic::Lexer<'a> for #name #gen {
            type Iterator = #iter_name<'a>;
            fn lex(input: &'a str) -> #iter_name<'a> {
                #iter_name {
                    input,
                    consumed: 0,
                }
            }
        }

        struct #iter_name<'a> {
            pub input: &'a str,
            pub consumed: usize,
        }

        impl<'a> Iterator for #iter_name<'a> {
            type Item = Result<(usize, #name #gen, usize), lexi_matic::Error>;
            fn next(&mut self) -> Option<Self::Item> {
                #dfa

                let start = self.consumed;
                let remaining = &self.input[self.consumed..];
                if remaining.is_empty() {
                    return None;
                }

                let (pat, len) = match lexi_matic::dfa_search_next(dfa, remaining) {
                    Some(t) => t,
                    None => return Some(Err(lexi_matic::Error(start))),
                };
                let t = match pat.as_u32() {
                    #(#matches)*
                    _ => unreachable!(),
                };
                self.consumed += len;
                Some(Ok((start, t, start + len)))
            }
        }
    };

    Ok(lexer_impl)
}
