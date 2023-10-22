use std::char;

use darling::FromAttributes;
use manyhow::manyhow;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Expr;
use syn::Field;
use syn::Type;
use syn::{parse::*, LitInt};

#[derive(Debug, FromAttributes)]
#[darling(attributes(pack))]
struct AsciiPackArgs {
    size: usize,
    pad_left: Option<char>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(pack_vec))]
struct AsciiPackVecArgs {
    until: Expr,
    pad_left: Option<char>,
    size: Option<LitInt>, // TODO: don't require this
}

impl Parse for AsciiPackVecArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        unimplemented!()
    }
}

fn extract_first_generic(ty: &Type) -> syn::Result<Type> {
    match ty {
        syn::Type::Path(type_path) => {
            let generics: std::result::Result<&syn::PathSegment, syn::Error> =
                match type_path.path.segments.first() {
                    Some(gen) => Ok(gen),
                    None => return Err(syn::Error::new(ty.span(), "Generic is required!")),
                };

            let generic_type = match &generics?.arguments {
                syn::PathArguments::None => {
                    return Err(syn::Error::new(ty.span(), "Generic is required!"))
                }
                syn::PathArguments::AngleBracketed(brack) => brack
                    .clone()
                    .args
                    .into_iter()
                    .map(|arg| match arg {
                        syn::GenericArgument::Type(t) => Some(t),
                        _ => None,
                    })
                    .filter(|t| t.is_some())
                    .flatten()
                    .last()
                    .unwrap(),
                syn::PathArguments::Parenthesized(paren) => paren
                    .inputs
                    .first()
                    .expect("expected a simple generic")
                    .clone(),
            };

            Ok(generic_type)
        }
        _ => Err(syn::Error::new(
            ty.span(),
            "Only path type generics are allowed!",
        )),
    }
}

fn generate_pack_tokens(
    mut from_ascii_tokens: TokenStream2,
    mut to_ascii_tokens: TokenStream2,
    args: AsciiPackArgs,
    field: &Field,
) -> syn::Result<(TokenStream2, TokenStream2)> {
    let name = &field.ident.clone().unwrap();
    let ty = &field.ty;
    let lit_name = name.to_string();

    let size_lit = args.size;
    //let size = size_lit.base10_parse::<usize>().unwrap();
    //let right_bound = left_bound + size - 1;

    let pad_left = args.pad_left.unwrap_or('0');

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
                format!("Size of item in {} was too large - item: {}, expected size: {}", #lit_name, substr, #size_lit)));
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

    Ok((from_ascii_tokens, to_ascii_tokens))
}

fn generate_pack_vec_tokens(
    mut from_ascii_tokens: TokenStream2,
    mut to_ascii_tokens: TokenStream2,
    args: AsciiPackVecArgs,
    field: &Field,
) -> syn::Result<(TokenStream2, TokenStream2)> {
    let ty = &field.ty;
    let generic_type = extract_first_generic(&ty)?;
    let name = &field.ident.clone().unwrap();
    let until = args.until;
    let has_size = args.size.is_some();
    let size = &args.size.unwrap_or(LitInt::new("99999", Span::call_site()));
    let pad_left = args.pad_left.unwrap_or('0');
    let lit_name = name.to_string();

    // TODO: this cannot be a fixed size, so we cannot use from_str here.
    // Instead, we must impl AsciiPack for primitives and then consume the buffer as necessary.

    from_ascii_tokens = quote! {
        #from_ascii_tokens
        let stop_fn = #until;
        let mut slice = match #has_size {
            true => &input[left_bound..=(left_bound + #size - 1).min(input.len() - 1)],
            false => &input[left_bound..]
        };
        while !stop_fn(&slice) {
            let value = #generic_type::from_ascii(&slice)?;
            match #has_size {
                true => {left_bound += #size;},
                false => {
                    println!("parsing: {}", slice);
                    left_bound += value.to_ascii().unwrap().len();
                }
            }
            let size_used = value.to_ascii()?.len();
            result.#name.push(value);
            slice = match #has_size {
                true => &input[left_bound..=(left_bound + #size - 1).min(input.len() - 1)],
                false => &input[left_bound..]
            };
        }
    };

    to_ascii_tokens = quote! {
        #to_ascii_tokens
        for item in &self.#name {
            let mut substr = item.to_ascii()?;
            if substr.len() > #size {
                return Err(AsciiPackError::Pack(
                    format!("Size of item in {} was too large - item: {}, expected size: {}", #lit_name, substr, #size)));
            }

            if !#has_size {
                result.push_str(&substr);
                continue;
            }

            let padding_size = #size - substr.len();
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
        }
    };

    Ok((from_ascii_tokens, to_ascii_tokens))
}

/// This macro is used to derive ascii format packing metadata and relevant functions to
/// pack and unpack structured, sized data from strongly sized ascii formats into native
/// rust types, bidirectionally.
///
/// Example:
///
/// ```
/// const TEST_ASCII: &str = "  EXAMPLETESTTESTTEST00120654012346543345delimeterabc";
///
/// #[derive(AsciiPack, PartialEq, Eq, Debug, Default)]
/// pub struct VecTest {
///     #[pack(size = 9, pad_left = ' ')]
///     pub string1: String,
///
///     #[pack_vec(size = 4, until = until::ascii_digit)]
///     pub string_vec: Vec<String>,
///
///     #[pack_vec(size = 4, until = until::starts_with("del"))]
///     pub usize_vec: Vec<usize>,
///
///     #[pack(size = 9)]
///     pub delimeter: String,
///
///     #[pack_vec(size = 1, until = until::empty)]
///     pub trailing_vec: Vec<char>,
/// }
/// ```
#[manyhow(proc_macro_derive(AsciiPack, attributes(pack, pack_ignore, pack_vec)))]
pub fn derive_ascii_pack(item: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(item)?;
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
                let args: AsciiPackArgs = FromAttributes::from_attributes(&field.attrs)?;
                let (from, to) =
                    generate_pack_tokens(from_ascii_tokens, to_ascii_tokens, args, &field)?;
                from_ascii_tokens = from;
                to_ascii_tokens = to;
            }
            "pack_vec" => {
                let args: AsciiPackVecArgs = FromAttributes::from_attributes(&field.attrs)?;
                let (from, to) =
                    generate_pack_vec_tokens(from_ascii_tokens, to_ascii_tokens, args, &field)?;
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
        impl ::ascii_pack::AsciiPack for #struc {
            fn from_ascii(input: &str) -> Result<Self, ::ascii_pack::AsciiPackError> {
                #from_ascii_tokens
            }

            fn to_ascii(&self) -> Result<String, ::ascii_pack::AsciiPackError> {
                #to_ascii_tokens
            }
        }
    };

    Ok(tokens.into())
}
