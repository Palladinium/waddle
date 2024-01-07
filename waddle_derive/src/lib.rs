#![recursion_limit = "128"]

extern crate proc_macro;

use std::{collections::HashMap, fmt, str::FromStr};

use itertools::{EitherOrBoth, Itertools};
use proc_macro2::{Literal, Span, TokenStream, TokenTree};
use proc_quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    bracketed, parenthesized,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Data, DeriveInput, Error, Ident, Result, Token,
};

#[proc_macro_derive(
    LineDefSpecial,
    attributes(udmf_special, doom_special, trigger_flags, udmf, doom)
)]
pub fn linedef_special_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ts = SpecialData::parse(input)
        .map(ToTokens::into_token_stream)
        .unwrap_or_else(|e| e.to_compile_error());

    proc_macro::TokenStream::from(ts)
}

struct SpecialData {
    linedef_special: Ident,
    udmf_special: Ident,
    doom_special: Ident,
    trigger_flags: Ident,
    specials: Vec<Special>,
}

impl SpecialData {
    fn parse(input: DeriveInput) -> Result<Self> {
        let mut udmf_value_buckets = HashMap::new();
        let mut doom_value_buckets = HashMap::new();

        let specials: Vec<_> = if let Data::Enum(en) = &input.data {
            en.variants
                .iter()
                .map(|variant| {
                    let fields: Vec<_> = variant
                        .fields
                        .iter()
                        .map(|field| field.ident.as_ref().cloned().unwrap())
                        .collect();

                    let udmf_value = parse_literal(parse_attribute(
                        "udmf",
                        &variant.attrs,
                        variant.ident.span(),
                    )?)?;

                    udmf_value_buckets
                        .entry(udmf_value)
                        .or_insert_with(Vec::new)
                        .push(variant.ident.span());

                    let doom_mappings: Vec<DoomMapping> =
                        collect_attributes::<DoomMapping>("doom", &variant.attrs)
                            .collect::<Result<Vec<_>>>()?;

                    for doom_mapping in doom_mappings.iter() {
                        doom_value_buckets
                            .entry(doom_mapping.value)
                            .or_insert_with(Vec::new)
                            .push(variant.ident.span());
                    }

                    Ok(Special {
                        ident: variant.ident.clone(),
                        udmf_value,
                        doom_mappings,
                        fields,
                    })
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            return Err(parse::Error::new(
                input.ident.span(),
                "Expected enum linedefspecial",
            ));
        };

        for (udmf_value, spans) in udmf_value_buckets.iter() {
            if spans.len() > 1 {
                return Err(parse::Error::new(
                    spans[1],
                    format!("Duplicate UDMF special with value {}", udmf_value),
                ));
            }
        }

        for (doom_value, spans) in doom_value_buckets.iter() {
            if spans.len() > 1 {
                return Err(parse::Error::new(
                    spans[1],
                    format!("Duplicate Doom special with value {}", doom_value),
                ));
            }
        }

        Ok(Self {
            linedef_special: input.ident.clone(),
            udmf_special: parse_attribute("udmf_special", &input.attrs, input.ident.span())?,
            doom_special: parse_attribute("doom_special", &input.attrs, input.ident.span())?,
            trigger_flags: parse_attribute("trigger_flags", &input.attrs, input.ident.span())?,

            specials,
        })
    }
}

struct Special {
    ident: Ident,
    udmf_value: i16,
    fields: Vec<Ident>,
    doom_mappings: Vec<DoomMapping>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum DoomMappingArg {
    Tag,
    Constant(i16),
}

impl Parse for DoomMappingArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Ident) {
            let ident: Ident = input.parse()?;
            if ident == Ident::new("tag", Span::call_site()) {
                Ok(DoomMappingArg::Tag)
            } else {
                Err(Error::new(ident.span(), "invalid ident"))
            }
        } else {
            Ok(DoomMappingArg::Constant(parse_literal(input.parse()?)?))
        }
    }
}

impl ToTokens for DoomMappingArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DoomMappingArg::Tag => tokens.append(Ident::new("tag", Span::call_site())),
            DoomMappingArg::Constant(value) => tokens.append(Literal::i16_unsuffixed(*value)),
        }
    }
}

struct AttrArg {
    key: Ident,
    equal_token: Token![=],
    value: TokenTree,
}

impl Parse for AttrArg {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            key: input.parse()?,
            equal_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for AttrArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.key.to_tokens(tokens);
        self.equal_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

struct AttrArgs {
    args: HashMap<String, TokenTree>,
    span: Span,
}

impl AttrArgs {
    pub fn try_get<T: Parse>(&self, key: &str) -> Result<Option<T>> {
        self.args
            .get(key)
            .map(|v| syn::parse2(std::iter::once(v.clone()).collect::<TokenStream>()))
            .transpose()
    }

    pub fn get<T: Parse>(&self, key: &str) -> Result<T> {
        self.try_get(key)?
            .ok_or_else(|| Error::new(self.span, format!("Missing attribute argument: {}", key)))
    }
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let args: Punctuated<AttrArg, Token![,]> =
            input.parse_terminated(AttrArg::parse, Token![,])?;

        Ok(Self {
            span: args.span(),
            args: args
                .into_iter()
                .map(|a| (a.key.to_string(), a.value))
                .collect(),
        })
    }
}

struct Tuple<T> {
    _paren_token: token::Paren,
    items: Punctuated<T, Token![,]>,
}

impl<T: Clone> Tuple<T> {
    fn to_vec(&self) -> Vec<T> {
        self.items.iter().cloned().collect()
    }
}

impl<T: Parse> Parse for Tuple<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents;

        Ok(Self {
            _paren_token: parenthesized!(contents in input),
            items: contents.parse_terminated(T::parse, Token![,])?,
        })
    }
}

struct Array<T> {
    _bracket_token: token::Bracket,
    items: Punctuated<T, Token![,]>,
}

impl<T: Clone> Array<T> {
    fn to_vec(&self) -> Vec<T> {
        self.items.iter().cloned().collect()
    }
}

impl<T: Parse> Parse for Array<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents;

        Ok(Self {
            _bracket_token: bracketed!(contents in input),
            items: contents.parse_terminated(T::parse, Token![,])?,
        })
    }
}

#[derive(PartialEq, Eq, Hash)]
struct DoomMapping {
    value: i16,
    arg_mappings: Vec<DoomMappingArg>,
    trigger_flags: Vec<Ident>,
}

impl Parse for DoomMapping {
    fn parse(input: ParseStream) -> Result<Self> {
        let args: AttrArgs = input.parse()?;

        let arg_mappings_tuple: Tuple<DoomMappingArg> = args.get("args")?;
        let flags_array: Array<Ident> = args.get("triggers")?;

        Ok(Self {
            value: parse_literal(args.get("id")?)?,
            arg_mappings: arg_mappings_tuple.to_vec(),
            trigger_flags: flags_array.to_vec(),
        })
    }
}

fn parse_literal<T>(lit: Literal) -> Result<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    lit.to_string()
        .parse()
        .map_err(|e| parse::Error::new(lit.span(), e))
}

fn parse_attribute<T: Parse>(name: &str, attributes: &[Attribute], span: Span) -> Result<T> {
    try_parse_attribute(name, attributes)?
        .ok_or_else(|| parse::Error::new(span, format!("Attribute `{}` missing", name)))
}

fn try_parse_attribute<T: Parse>(name: &str, attributes: &[Attribute]) -> Result<Option<T>> {
    attributes
        .iter()
        .find(|a| a.path().is_ident(&Ident::new(name, Span::call_site())))
        .map(|a| syn::parse2::<T>(a.meta.require_list()?.tokens.clone()))
        .transpose()
}

fn collect_attributes<'a, T: Parse>(
    name: &'a str,
    attributes: &'a [Attribute],
) -> impl Iterator<Item = Result<T>> + 'a {
    attributes
        .iter()
        .filter(move |a| a.path().is_ident(&Ident::new(name, Span::call_site())))
        .map(|a| syn::parse2::<T>(a.meta.require_list()?.tokens.clone()))
}

impl ToTokens for SpecialData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.gen_from_udmf_tokens(tokens);
        self.gen_into_udmf_tokens(tokens);
        self.gen_from_doom_tokens(tokens);
    }
}

impl SpecialData {
    fn gen_from_udmf_tokens(&self, tokens: &mut TokenStream) {
        let udmf_special = &self.udmf_special;
        let linedef_special = &self.linedef_special;

        let match_arms = self.specials.iter().map(|special| {
            let udmf_value = &special.udmf_value;
            let variant = &special.ident;
            let field_exprs = special.fields.iter().enumerate().map(|(i, field)| {
                quote! { #field: udmf.args[#i] }
            });
            let fields_len = special.fields.len();
            let extra_fields_checks = (fields_len..5).map(|i| {
                quote! {
                    if udmf.args[#i] != 0 {
                        return Err(udmf);
                    }
                }
            });

            quote! {
                #udmf_value => {
                    #(#extra_fields_checks)*
                    Ok(#linedef_special::#variant { #(#field_exprs),* })
                }
            }
        });

        tokens.extend(quote! {
            impl std::convert::TryFrom<#udmf_special> for #linedef_special {
                type Error = #udmf_special;

                fn try_from(udmf: #udmf_special) -> Result<Self, Self::Error> {
                    match udmf.value {
                        #(#match_arms,)*
                        _ => Err(udmf),
                    }
                }
            }

        });
    }

    fn gen_into_udmf_tokens(&self, tokens: &mut TokenStream) {
        let udmf_special = &self.udmf_special;
        let linedef_special = &self.linedef_special;

        let match_arms = self.specials.iter().map(|special| {
            let udmf_value = &special.udmf_value;
            let variant = &special.ident;
            let fields = &special.fields;
            let field_exprs = special
                .fields
                .iter()
                .map(|field| quote! { #field })
                .pad_using(5, |_| quote! { 0 });

            quote! {
                #linedef_special::#variant { #(#fields),* } => #udmf_special::new(#udmf_value, [#(#field_exprs),*])
            }});

        tokens.extend(quote! {
            impl From<#linedef_special> for #udmf_special {
                fn from(special: #linedef_special) -> Self {
                    match special {
                        #(#match_arms,)*
                    }
                }
            }
        });
    }

    fn gen_from_doom_tokens(&self, tokens: &mut TokenStream) {
        let doom_special = &self.doom_special;
        let linedef_special = &self.linedef_special;
        let trigger_flags = &self.trigger_flags;

        let match_arms = self.specials.iter().map(|special| {
            let variant = &special.ident;

            special.doom_mappings.iter().map(move |doom_mapping| {
                let doom_value = doom_mapping.value;
                let fields = special
                    .fields
                    .iter()
                    .zip_longest(doom_mapping.arg_mappings.iter())
                    .map(|e| match e {
                        EitherOrBoth::Left(f) => quote! { #f: 0 },
                        EitherOrBoth::Right(_) => panic!(),
                        EitherOrBoth::Both(f, v) => quote! { #f: #v },
                    });

                let flags = doom_mapping
                    .trigger_flags
                    .iter()
                    .map(|f| quote! { #f: true });

                quote! {
                    #doom_value => Ok((
                        #linedef_special::#variant { #(#fields,)* },
                        #trigger_flags {  #(#flags,)* ..#trigger_flags::default() },
                    ))
                }
            })
        });

        tokens.extend(quote! {
            impl std::convert::TryFrom<#doom_special> for (#linedef_special, #trigger_flags) {
                type Error = #doom_special;

                fn try_from(doom: #doom_special) -> Result<Self, Self::Error> {
                    let tag = doom.tag;
                    match doom.value {
                        #(#(#match_arms,)*)*

                        _ => Err(doom),
                    }
                }
            }

        });
    }
}
