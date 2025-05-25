use error::MacroError;
use proc_macro::{TokenStream, TokenTree};
mod error;
use TokenTree as TT;
use std::fmt::Write;

macro_rules! error {
    ($arr:expr, $($arg:tt)*) => {
        return Err(MacroError::slice($arr, format!($($arg)*)))
    };
}

macro_rules! error_single {
    ($tt:expr, $($arg:tt)*) => {
        return Err(MacroError::start_end($tt, $tt, format!($($arg)*)))
    };
}

#[proc_macro_derive(Quicksilver)]
pub fn derive_quicksilver(input: TokenStream) -> TokenStream {
    return match inner(input) {
        Ok(tt) => tt,
        Err(err) => err.to_compile_error(),
    };
}

fn inner(item: TokenStream) -> Result<TokenStream, MacroError> {
    dbg!(&item);
    let mut iter = item.into_iter();
    match (iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(TT::Ident(s)), Some(TT::Ident(name)), Some(TT::Group(fields)), None) => {
            assert_eq!("struct", s.to_string());
            let name = name.to_string();
            let fields = parse_fields(fields.stream())?;
            generate_impl(name, fields)
        }
        _ => panic!("Unsupported struct shape."),
    }
}

fn generate_impl(name: String, fields: Vec<Field>) -> Result<TokenStream, MacroError> {
    let result = &mut String::new();
    write!(result, r#"
impl Reflection for {name} {{
    const MIRROR: &'static Struct = &Struct {{
        name: "{name}",
        size: size_of::<Self>(),
        align: align_of::<Self>(),
        fields: &[
            Field {{
                name: "x",
                ty: Type::I32,
                offset: mem::offset_of!(Self, x),
            }},
            Field {{
                name: "y",
                ty: Type::I32,
                offset: mem::offset_of!(Self, y),
            }},
        ],
    }};
}}
"#).unwrap();
    Ok("fn answer() -> u32 { 42 }".parse().unwrap())
}

struct Field {
    name: String,
    ty: String,
}

fn parse_fields(input: TokenStream) -> Result<Vec<Field>, MacroError> {
    let mut iter = input.into_iter();
    let mut buffer = Vec::new();
    let mut result = Vec::new();

    let mut current = iter.next();
    while matches!(current, Some(_)) {
        if matches!(current, Some(TT::Punct(ref comma)) if comma.as_char() == ',') {
            result.push(parse_field(&buffer)?);
            buffer.clear();
        } else {
            buffer.push(current.unwrap());
        }

        current = iter.next();
    }

    Ok(result)
}

fn parse_field(buffer: &[TokenTree]) -> Result<Field, MacroError> {
    dbg!(buffer);
    let mut iter = buffer.iter();
    match (iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(TT::Ident(name)), Some(tt_colon @ TT::Punct(colon)), Some(TT::Ident(ty)), None) => {
            if colon.as_char() != ':' {
                error_single!(tt_colon, "Expected ':'")
            }
            let name = name.to_string();
            let ty = ty.to_string();
            Ok(Field { name, ty })
        }
        _ => error!(
            &[buffer[0].clone(), buffer.last().unwrap().clone()],
            "Can't parse."
        ),
    }
}
