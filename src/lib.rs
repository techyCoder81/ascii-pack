use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::*, LitInt, LitChar};

mod kw {
    syn::custom_keyword!(size);
    syn::custom_keyword!(pad_left);
}

struct AsciiPackArgs {
    size: LitInt,
    pad_left: Option<LitChar>
}

impl Parse for AsciiPackArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: kw::size = input.parse().expect("size is a required attribute!");
        let _: syn::Token![=] = input.parse().expect("Equals sign was missing!");
        let size: syn::LitInt = input.parse().expect("size field failed to parse!");

        if input.is_empty() {
            return Ok(Self { size: size, pad_left: None });
        }
        input.parse::<syn::Token![,]>().expect("Expected a comma!");
        input.parse::<kw::pad_left>().expect("Expected pad_left argument!");
        input.parse::<syn::Token![=]>().expect("Expected an equals sign!");
        let pad_left: syn::LitChar = input.parse().expect("Expected a literal character!");

        Ok(Self { size: size, pad_left: Some(pad_left)})
    }
}

#[proc_macro_derive(AsciiPack, attributes(pack))]
pub fn derive_helper_attr(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let data = match input.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(_) => panic!("not supported for enums"),
        syn::Data::Union(_) => panic!("not supported for unions"),
    };

    let struc = input.ident;

    let mut from_ascii_tokens = quote! {};
    let mut to_ascii_tokens = quote! {
        let mut result = String::new();
    };

    let mut left_bound = 0usize;
    for field in data.fields.iter() {
        for attr in field.attrs.iter() {
            if !attr.meta.path().is_ident("pack") {
                continue;
            }
            
            let args = attr.parse_args::<AsciiPackArgs>().unwrap();

            let size_lit = args.size;
            let size = size_lit.base10_parse::<usize>().unwrap();
            let right_bound = left_bound + size - 1;

            let pad_left = args.pad_left.unwrap_or(LitChar::new('0', Span::call_site()));
            let name = &field.ident.clone().unwrap();
            let ty = &field.ty;

            from_ascii_tokens = quote! {
                #from_ascii_tokens
                #name: #ty::from_str(&input[#left_bound..=#right_bound]).unwrap(),
            };

            to_ascii_tokens = quote! {
                #to_ascii_tokens
                let mut substr = &self.#name.to_string();
                if substr.len() > #size_lit {
                    return Err(MessageFormatParseError {
                        error: format!("Size of field {} was too large: {}", "#name", #size_lit)
                    });
                }

                let padding_size = #size_lit - substr.len();
                if padding_size > 0 {
                    let mut pad_str = String::new();
                    for _ in 0..padding_size {
                        pad_str.push(#pad_left);
                    }
                    pad_str.push_str(&substr);
                    result.push_str(&pad_str)
                } else {
                    result.push_str(&substr);
                }
            };

            left_bound += size;
        }
    }

    to_ascii_tokens = quote! {
        #to_ascii_tokens
        return Ok(result)
    };

    let tokens = quote! {
        #[derive(Debug)]
        struct MessageFormatParseError {
            error: String
        }

        trait AsciiPack {
            fn from_ascii(input: &str) -> Result<#struc, MessageFormatParseError>;
            fn to_ascii(&self) -> Result<String, MessageFormatParseError>;
        }

        impl AsciiPack for #struc {
            fn from_ascii(input: &str) -> Result<#struc, MessageFormatParseError> {
                use std::str::FromStr;
                Ok(#struc {
                    #from_ascii_tokens
                })
            }
            
            fn to_ascii(&self) -> Result<String, MessageFormatParseError> {
                #to_ascii_tokens
            }
        }
    };
    
    return tokens.into()
}