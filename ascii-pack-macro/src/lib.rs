use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;
use syn::Type;
use syn::{parse::*, Attribute, ExprClosure, LitChar, LitInt, LitStr};

mod kw {
    syn::custom_keyword!(size);
    syn::custom_keyword!(pad_left);
    syn::custom_keyword!(until);
}

struct AsciiPackArgs {
    size: Option<LitInt>,
    pad_left: Option<LitChar>,
}

struct AsciiPackVecArgs {
    item: Type,
    until: ExprClosure,
    size: LitInt, // TODO: don't require this
}

fn try_parse_pad_left(input: &ParseStream) -> Option<LitChar> {
    match input.parse::<kw::pad_left>() {
        Ok(_) => {}
        Err(_) => return None,
    };
    input
        .parse::<syn::Token![=]>()
        .expect("Expected an equals for pad_left!");
    let pad_left: syn::LitChar = input
        .parse()
        .expect("Expected a character literal for pad_left!");

    Some(pad_left)
}

fn try_parse_size(input: &ParseStream) -> Option<LitInt> {
    match input.parse::<kw::size>() {
        Ok(_) => {}
        Err(_) => return None,
    };
    input
        .parse::<syn::Token![=]>()
        .expect("Expected an equals for size!");
    let size: syn::LitInt = input
        .parse()
        .expect("Expected an integer literal for size!");

    Some(size)
}

impl Parse for AsciiPackArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut size = None;
        let mut pad_left = None;
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

            if !parsed {
                // the stream wasn't empty, but we couldn't parse out any fields either!
                panic!("Failed to parse attributes: {}", input)
            }
        }

        Ok(Self { size, pad_left })
    }
}

impl Parse for AsciiPackVecArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        unimplemented!()
    }
}

fn generate_pack_tokens(
    mut from_ascii_tokens: TokenStream,
    mut to_ascii_tokens: TokenStream,
    field: &Field,
    attr: &Attribute,
) -> (TokenStream, TokenStream) {
    let args = attr.parse_args::<AsciiPackArgs>().unwrap();
    let name = &field.ident.clone().unwrap();
    let ty = &field.ty;

    let size_lit = args.size;
    //let size = size_lit.base10_parse::<usize>().unwrap();
    //let right_bound = left_bound + size - 1;

    let pad_left = args
        .pad_left
        .unwrap_or(LitChar::new('0', Span::call_site()));

    from_ascii_tokens = quote! {
        #from_ascii_tokens
        result.#name = #ty::from_ascii(&input[left_bound..=(left_bound + #size_lit - 1)])?;
        left_bound += #size_lit;
    };

    to_ascii_tokens = quote! {
        #to_ascii_tokens
        let mut substr = &self.#name.to_ascii()?;
        if substr.len() > #size_lit {
            return Err(AsciiPackError::Pack(
                format!("Size of field {} was too large: {}", "#name", #size_lit)));
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

    (from_ascii_tokens, to_ascii_tokens)
}

fn generate_pack_vec_tokens(
    mut from_ascii_tokens: TokenStream,
    mut to_ascii_tokens: TokenStream,
    field: &Field,
    attr: &Attribute,
) -> (TokenStream, TokenStream) {
    let args = attr.parse_args::<AsciiPackVecArgs>().unwrap();
    let name = &field.ident.clone().unwrap();
    let ty = &field.ty;

    let item = args.item;
    let until = args.until;
    let size = args.size;
    // TODO: this cannot be a fixed size, so we cannot use from_str here.
    // Instead, we must impl AsciiPack for primitives and then consume the buffer as necessary.

    from_ascii_tokens = quote! {
        #from_ascii_tokens
        while !#until {
            result.push(#ty::from_ascii(&input[left_bound..=(left_bound + #size - 1)])?);
            left_bound += #size;
        }
    };

    to_ascii_tokens = quote! {
        #to_ascii_tokens
        for item in &self.name.iter() {
            let mut substr = item.to_ascii()?;
            if substr.len() != #size {
                return Err(AsciiPackError::Pack(
                    format!("Size of field {} was too large: {}", "#name", #size)));
            }

            result.push_str(&substr);
        }
    };

    (from_ascii_tokens, to_ascii_tokens)
}

#[proc_macro_derive(AsciiPack, attributes(pack, pack_ignore))]
pub fn derive_helper_attr(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
            .find(|attr| {
                attr.meta.path().is_ident("pack")
                    || attr.meta.path().is_ident("pack_ignore")
                    || attr.meta.path().is_ident("pack_vec")
            })
            .expect("A `pack`, `pack_ignore`, or `pack_vec` attribute is required on all fields!");

        match attr.meta.path().get_ident().unwrap().to_string().as_str() {
            "pack_ignore" => {
                // leave the default value
                continue;
            }
            "pack" => {
                let (from, to) =
                    generate_pack_tokens(from_ascii_tokens, to_ascii_tokens, &field, attr);
                from_ascii_tokens = from;
                to_ascii_tokens = to;
            }
            "pack_vec" => {
                let (from, to) =
                    generate_pack_vec_tokens(from_ascii_tokens, to_ascii_tokens, &field, attr);
                from_ascii_tokens = from;
                to_ascii_tokens = to;
            }
            _ => {}
        }
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
        use ::ascii_pack::AsciiPackError;

        impl ::ascii_pack::AsciiPack for #struc {
            fn from_ascii(input: &str) -> Result<Self, AsciiPackError> {
                #from_ascii_tokens
            }

            fn to_ascii(&self) -> Result<String, AsciiPackError> {
                #to_ascii_tokens
            }
        }
    };

    return tokens.into();
}
