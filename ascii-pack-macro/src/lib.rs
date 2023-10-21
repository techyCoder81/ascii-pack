#![feature(never_type)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::*, LitInt, LitChar, ExprClosure, LitStr};

mod kw {
    syn::custom_keyword!(size);
    syn::custom_keyword!(pad_left);
    syn::custom_keyword!(until);
}

struct AsciiPackArgs {
    size: Option<LitInt>,
    pad_left: Option<LitChar>,
    until: Option<ExprClosure> // TODO: manage repeats
}

fn try_parse_until(input: &ParseStream) -> Option<ExprClosure> {
    match input.parse::<kw::until>() {
        Ok(_) => {},
        Err(_) => return None
    };
    input.parse::<syn::Token![=]>().expect("Expected an equals for until!");
    let until: syn::ExprClosure = input.parse().expect("Expected a closure literal for until!");
    unimplemented!("arbitrary repeated sections are not yet implemented!");
    //return Some(max_repeats);
}

fn try_parse_pad_left(input: &ParseStream) -> Option<LitChar> {
    match input.parse::<kw::pad_left>() {
        Ok(_) => {},
        Err(_) => return None
    };
    input.parse::<syn::Token![=]>().expect("Expected an equals for pad_left!");
    let pad_left: syn::LitChar = input.parse().expect("Expected a character literal for pad_left!");

    return Some(pad_left);
}

fn try_parse_size(input: &ParseStream) -> Option<LitInt> {
    match input.parse::<kw::size>() {
        Ok(_) => {},
        Err(_) => return None
    };
    input.parse::<syn::Token![=]>().expect("Expected an equals for size!");
    let size: syn::LitInt = input.parse().expect("Expected an integer literal for size!");

    return Some(size);
}

impl Parse for AsciiPackArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut size = None;
        let mut pad_left = None;
        let mut until = None;
        let mut parsed = false;

        while !input.is_empty() {
            parsed = false;

            let _ = input.parse::<syn::Token![,]>();

            if let Some(val) = try_parse_size(&input) {
                if size.is_some() {
                    panic!("Atrributes may only be specified once per field!")
                }
                size = Some(val);
                parsed = true;
            }

            if let Some(val) = try_parse_pad_left(&input) {
                if pad_left.is_some() {
                    panic!("Atrributes may only be specified once per field!");
                }
                pad_left = Some(val);
                parsed = true;
            }

            if let Some(val) = try_parse_until(&input) {
                if until.is_some() {
                    panic!("Atrributes may only be specified once per field!");
                }
                until = Some(val);
                parsed = true;
            }

            if !parsed {
                // the stream wasn't empty, but we couldn't parse out any fields either!
                panic!("Failed to parse attributes: {}", input)
            }
        }

        Ok(Self { size, pad_left, until })
    }

    
}

#[proc_macro_derive(AsciiPack, attributes(pack, pack_ignore))]
pub fn derive_helper_attr(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let data = match input.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(_) => panic!("not supported for enums"),
        syn::Data::Union(_) => panic!("not supported for unions"),
    };

    let struc = input.ident;

    let mut from_ascii_tokens = quote! {
        use std::str::FromStr;
        let mut left_bound = 0usize;
        let mut result = #struc::default();
    };
    let mut to_ascii_tokens = quote! {
        let mut result = String::new();
    };

    for field in data.fields.iter() {
        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.meta.path().is_ident("pack") || attr.meta.path().is_ident("pack_ignore"))
            .expect("A `pack` or `pack_ignore` attribute is required on all fields!");

        let name = &field.ident.clone().unwrap();
        let ty = &field.ty;

        if attr.meta.path().is_ident("pack_ignore") {
            // leave the default value
            continue;
        }

        let args = attr.parse_args::<AsciiPackArgs>().unwrap();

        let size_lit = args.size;
        //let size = size_lit.base10_parse::<usize>().unwrap();
        //let right_bound = left_bound + size - 1;

        let pad_left = args.pad_left.unwrap_or(LitChar::new('0', Span::call_site()));

        from_ascii_tokens = quote! {
            #from_ascii_tokens
            result.#name = #ty::from_str(&input[left_bound..=(left_bound + #size_lit - 1)]).unwrap();
            left_bound += #size_lit;
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
    }

    from_ascii_tokens = quote! {
        #from_ascii_tokens
        return Ok(result)
    };

    to_ascii_tokens = quote! {
        #to_ascii_tokens
        return Ok(result)
    };

    let tokens = quote! {
        use ::ascii_pack::MessageFormatParseError;

        impl ::ascii_pack::AsciiPack for #struc {
            fn from_ascii(input: &str) -> Result<Self, MessageFormatParseError> {
                #from_ascii_tokens
            }
            
            fn to_ascii(&self) -> Result<String, MessageFormatParseError> {
                #to_ascii_tokens
            }
        }

        impl std::str::FromStr for #struc {
            type Err = MessageFormatParseError;
        
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                #struc::from_ascii(s)
            }
        }
        
        impl ToString for #struc {
            fn to_string(&self) -> String {
                self.to_ascii().unwrap()
            }
        }
    };
    
    return tokens.into()
}
