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
struct PackArgs {
    size: usize,
    pad_left: Option<char>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(pack_vec))]
struct PackVecArgs {
    until: Expr,
    pad_left: Option<char>,
    size: Option<LitInt>, // TODO: don't require this
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(pack_static))]
struct PackStaticArgs {
    text: String,
}

impl Parse for PackVecArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        unimplemented!()
    }
}

/// Utility function to extract the type `T` from a single-type
/// generic such as `Vec<T>`. This assumes that the field is defined
/// literally as `Vec<T>`, with no type aliasing, as a type aliased
/// generic (`type MyVec = Vec<T>;`) will not work with this logic.
fn extract_first_generic(ty: &Type) -> syn::Result<Type> {
    match ty {
        syn::Type::Path(type_path) => {
            let generics: std::result::Result<&syn::PathSegment, syn::Error> =
                match type_path.path.segments.first() {
                    Some(gen) => Ok(gen),
                    None => return Err(syn::Error::new(ty.span(), "Generic is required!")),
                };

            let generic_type = match &generics?.arguments {
                syn::PathArguments::None => None,
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
                    .last(),
                syn::PathArguments::Parenthesized(paren) => paren.inputs.first().clone().cloned(),
            };

            match generic_type {
                Some(t) => Ok(t),
                None => Err(syn::Error::new(
                    ty.span(),
                    "Failed to parse single generic type! The type of this field must be of the literal form `Vec<T>`. Type aliasing is not supported.",
                )),
            }
        }
        _ => Err(syn::Error::new(
            ty.span(),
            "Only path type generics are allowed!",
        )),
    }
}

/// Generates the `to_ascii` and `from_ascii` tokens
/// for pack_vec fields
fn generate_pack_tokens(
    mut from_ascii_tokens: TokenStream2,
    mut to_ascii_tokens: TokenStream2,
    args: PackArgs,
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

/// Generates the `to_ascii` and `from_ascii` tokens
/// for pack_vec fields
fn generate_pack_vec_tokens(
    mut from_ascii_tokens: TokenStream2,
    mut to_ascii_tokens: TokenStream2,
    args: PackVecArgs,
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
                    left_bound += value.to_ascii()?.len();
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

/// Generates the `to_ascii` and `from_ascii` tokens
/// for pack_static fields
fn generate_pack_static_tokens(
    mut from_ascii_tokens: TokenStream2,
    mut to_ascii_tokens: TokenStream2,
    args: PackStaticArgs,
) -> syn::Result<(TokenStream2, TokenStream2)> {
    let static_value = args.text;
    let size = static_value.len();

    from_ascii_tokens = quote! {
        #from_ascii_tokens
        // no need to set the field - default will be fine.
        left_bound += #size;
    };

    to_ascii_tokens = quote! {
        #to_ascii_tokens
        // push the static string value onto the output.
        result.push_str(#static_value);
    };

    Ok((from_ascii_tokens, to_ascii_tokens))
}

/// Process the given field and output the to_ascii
/// and from_ascii tokens.
///
/// Note: this Field may include attributes from other macros
/// invoked by the user that are not relevant to AsciiPack.
fn process_field(
    mut from_ascii_tokens: TokenStream2,
    mut to_ascii_tokens: TokenStream2,
    field: &Field,
) -> syn::Result<(TokenStream2, TokenStream2)> {
    let mut already_parsed = false;
    for attr in field.attrs.iter() {
        let name = attr.meta.path().require_ident()?.to_string();
        let matched = match name.as_str() {
            "pack_ignore" => {
                // leave the default value
                true
            }
            "pack" => {
                let args: PackArgs = FromAttributes::from_attributes(&field.attrs)?;
                let (from, to) =
                    generate_pack_tokens(from_ascii_tokens, to_ascii_tokens, args, &field)?;
                from_ascii_tokens = from;
                to_ascii_tokens = to;
                true
            }
            "pack_vec" => {
                let args: PackVecArgs = FromAttributes::from_attributes(&field.attrs)?;
                let (from, to) =
                    generate_pack_vec_tokens(from_ascii_tokens, to_ascii_tokens, args, &field)?;
                from_ascii_tokens = from;
                to_ascii_tokens = to;
                true
            }
            "pack_static" => {
                let args: PackStaticArgs = FromAttributes::from_attributes(&field.attrs)?;
                let (from, to) =
                    generate_pack_static_tokens(from_ascii_tokens, to_ascii_tokens, args)?;
                from_ascii_tokens = from;
                to_ascii_tokens = to;
                true
            }
            _ => false, // attribute not relevant to ascii pack
        };

        if matched && already_parsed {
            return Err(syn::Error::new(
                field.span(),
                "Exactly one AsciiPack attribute is required on all fields!",
            ));
        }

        already_parsed = true;
    }

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
#[manyhow(proc_macro_derive(AsciiPack, attributes(pack, pack_ignore, pack_vec, pack_static)))]
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
        let (from, to) = process_field(from_ascii_tokens, to_ascii_tokens, field)?;
        from_ascii_tokens = from;
        to_ascii_tokens = to;
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
