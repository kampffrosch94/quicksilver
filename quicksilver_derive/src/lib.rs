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

#[proc_macro_derive(Quicksilver, attributes(quicksilver))]
pub fn derive_quicksilver(input: TokenStream) -> TokenStream {
    return match inner(input) {
        Ok(tt) => tt,
        Err(err) => err.to_compile_error(),
    };
}

fn inner(item: TokenStream) -> Result<TokenStream, MacroError> {
    let mut iter = item.into_iter();
    match (iter.next(), iter.next(), iter.next(), iter.next()) {
        // regular old struct
        (Some(TT::Ident(s)), Some(TT::Ident(name)), Some(TT::Group(fields)), None) => {
            assert_eq!("struct", s.to_string());
            let name = name.to_string();
            let fields = parse_fields(fields.stream())?;
            generate_impl(name, fields)
        }
        // tuple struct
        (
            Some(TT::Ident(s)),
            Some(TT::Ident(name)),
            Some(TT::Group(fields)),
            Some(TT::Punct(_)),
        ) => {
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
    write!(
        result,
        r#"
impl Reflection for {name} {{
    const MIRROR: Type = Type::Struct(&Struct {{
        name: "{name}",
        size: size_of::<Self>(),
        align: align_of::<Self>(),
        fields: &["#
    )
    .unwrap();

    for (i, field) in fields.into_iter().enumerate() {
        generate_field(
            result,
            &field.name.unwrap_or_else(|| format!("{i}")),
            &field.ty,
        );
    }

    write!(
        result,
        r#"
        ],
    }});
}}
"#
    )
    .unwrap();
    Ok(result.parse().unwrap())
}

fn generate_field(result: &mut String, name: &str, ty: &str) {
    write!(
        result,
        r#"
Field {{
    name: "{name}",
    ty: {ty},
    offset: ::std::mem::offset_of!(Self, {name}),
}},"#
    )
    .unwrap()
}

struct Field {
    name: Option<String>,
    ty: String,
}

fn parse_fields(input: TokenStream) -> Result<Vec<Field>, MacroError> {
    let mut iter = input.into_iter();
    let mut buffer = Vec::new();
    let mut result = Vec::new();

    let mut current = iter.next();
    // because types can use commas inside themselves like `HashMap<X,Y>`
    // we need to check that we are not inside the type of a field via nesting level
    let mut level = 0;
    while matches!(current, Some(_)) {
        if matches!(current, Some(TT::Punct(ref comma)) if comma.as_char() == ',') && level == 0 {
            result.push(parse_field(&buffer)?);
            buffer.clear();
        } else {
            if matches!(current, Some(TT::Punct(ref c)) if c.as_char() == '<') {
                level += 1;
            } else if matches!(current, Some(TT::Punct(ref c)) if c.as_char() == '>') {
                level -= 1;
            }
            buffer.push(current.unwrap());
        }

        current = iter.next();
    }
    if buffer.len() > 0 {
        result.push(parse_field(&buffer)?);
    }

    Ok(result)
}

fn parse_field(mut buffer: &[TokenTree]) -> Result<Field, MacroError> {
    let mut skip = false;
    if matches!(&buffer[0], TT::Punct(c) if c.as_char() == '#') {
        if let TT::Group(attribute_group) = &buffer[1] {
            for a in chunked(attribute_group.stream(), 2) {
                let [name, group] = &a[..] else {
                    unreachable!()
                };
                if matches!(name, TT::Ident(s) if s.to_string() == "quicksilver") {
                    let TT::Group(group) = group else {
                        panic!("Expected group.")
                    };
                    for attr in group.stream() {
                        if matches!(attr, TT::Ident(s) if s.to_string() == "skip") {
                            skip = true;
                        }
                    }
                }
            }
        }
        buffer = &buffer[2..];
    }

    let mut iter = buffer.iter();

    match (iter.next(), iter.next(), iter.next()) {
        (Some(TT::Ident(ty)), None, None) => {
            let name = None;
            let ty = parse_type(&buffer, &ty.to_string(), skip)?;
            Ok(Field { name, ty })
        }
        (Some(TT::Ident(name)), Some(tt_colon @ TT::Punct(colon)), Some(TT::Ident(ty))) => {
            if colon.as_char() != ':' {
                error_single!(tt_colon, "Expected ':'")
            }
            let name = Some(name.to_string());
            let buffer = &buffer[2..];
            let ty = parse_type(&buffer, &ty.to_string(), skip)?;
            Ok(Field { name, ty })
        }
        _ => error!(
            &[buffer[0].clone(), buffer.last().unwrap().clone()],
            "Quicksilver can't parse this."
        ),
    }
}

fn parse_type(buffer: &[TokenTree], ty: &str, skip: bool) -> Result<String, MacroError> {
    Ok(if buffer.len() == 1 {
        if skip {
            error!(
                &[buffer[0].clone(), buffer.last().unwrap().clone()],
                "Quicksilver can't skip this field."
            )
        }
        assert!(!skip, "Can't skip this field.");
        let mut result = String::new();
        result.push_str(ty);
        result.push_str("::TYPE");
        result
    } else {
        let mut result = String::new();
        result.push_str(ty);
        result.push_str("::");
        for token in &buffer[1..] {
            result.push_str(&token.to_string());
        }
        if skip {
            result.push_str("::EMPTY");
        } else {
            result.push_str("::TYPE");
        }
        result
    })
}

fn chunked<I>(a: impl IntoIterator<Item = I>, chunk_size: usize) -> impl Iterator<Item = Vec<I>> {
    let mut a = a.into_iter();
    std::iter::from_fn(move || {
        Some(a.by_ref().take(chunk_size).collect())
            .filter(|chunk: &Vec<_>| chunk.len() == chunk_size)
    })
}
