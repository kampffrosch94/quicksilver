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
    write!(
        result,
        r#"
impl Reflection for {name} {{
    const MIRROR: &'static Struct = &Struct {{
        name: "{name}",
        size: size_of::<Self>(),
        align: align_of::<Self>(),
        fields: &["#
    )
    .unwrap();
    for field in &fields {
        generate_field(result, field);
    }

    write!(
        result,
        r#"
        ],
    }};
}}
"#
    )
    .unwrap();
    Ok(result.parse().unwrap())
}

fn generate_field(result: &mut String, field: &Field) {
    let name = &field.name;
    let ty = &field.ty;
    write!(
        result,
        r#"
Field {{
    name: "{name}",
    ty: {ty},
    offset: mem::offset_of!(Self, {name}),
}},"#
    )
    .unwrap()
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
    match (iter.next(), iter.next(), iter.next()) {
        (Some(TT::Ident(name)), Some(tt_colon @ TT::Punct(colon)), tt_ty @ Some(TT::Ident(ty))) => {
            if colon.as_char() != ':' {
                error_single!(tt_colon, "Expected ':'")
            }
            let name = name.to_string();
            let buffer = &buffer[2..];
            let ty = parse_type(&buffer, &ty.to_string())?;
            Ok(Field { name, ty })
        }
        _ => error!(
            &[buffer[0].clone(), buffer.last().unwrap().clone()],
            "Quicksilver can't parse this."
        ),
    }
}

fn parse_type(buffer: &[TokenTree], ty: &str) -> Result<String, MacroError> {
    Ok(if buffer.len() == 1 {
        match ty {
            "i32" => "Type::I32".to_string(),
            "u32" => "Type::U32".to_string(),
            "f32" => "Type::F32".to_string(),
            "String" => "Type::String".to_string(),
            s => format!("Type::Struct({s}::MIRROR)"),
        }
    } else {
        match ty {
            "Vec" => {
                // parse what is inside Vec< .. >
                let TT::Ident(ref ident) = buffer[2] else {
                    error_single!(&buffer[2], "expected Type identifier")
                };
                let inner_name = ident.to_string();
                let inner = parse_type(&buffer[2..(buffer.len() - 1)], &inner_name)?;
                format!(
                    r#"
Type::Vec(VecType {{
    element: &{inner},
    vtable: VecVtableCreator::<{inner_name}>::VTABLE,
}})"#
                )
            }
            _ => error!(
                &[buffer[0].clone(), buffer.last().unwrap().clone()],
                "Quicksilver does not support this type."
            ),
        }
    })
}
