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

#[allow(unused)]
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

#[derive(Debug)]
enum Repr {
    Rust,
    C,
}

fn inner(item: TokenStream) -> Result<TokenStream, MacroError> {
    let mut iter = item.into_iter().peekable();

    let mut repr = Repr::Rust;
    loop {
        if matches!(iter.peek(), Some(TT::Ident(ident))
                if ["pub", "pub(crate)"].contains(&ident.to_string().as_str()))
        {
            let _ = iter.next();
            continue;
        }

        if matches!(iter.peek(), Some(TT::Punct(hashtag))
                if hashtag.as_char() == '#')
        {
            let _hashtag = iter.next();
            let group = iter.next();
            match group {
                Some(TT::Group(group)) => {
                    let mut iter = group.stream().into_iter();
                    match (iter.next(), iter.next(), iter.next()) {
                        (Some(TT::Ident(repr_ident)), Some(TT::Group(repr_group)), None)
                            if repr_ident.to_string() == "repr" =>
                        {
                            let mut iter = repr_group.stream().into_iter();
                            match (iter.next(), iter.next()) {
                                (Some(TT::Ident(c)), None) if c.to_string() == "C" => {
                                    repr = Repr::C; // <= the thing I want to know
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            continue;
        }
        break;
    }

    match (
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
    ) {
        // regular old struct or enum
        (
            Some(ref keyword @ TT::Ident(ref s)),
            Some(TT::Ident(name)),
            Some(TT::Group(fields)),
            None,
            None,
        ) => match s.to_string().as_str() {
            "struct" => {
                let name = name.to_string();
                let fields = parse_fields(fields.stream())?;
                generate_struct_impl(name, fields)
            }
            "enum" => {
                let name = name.to_string();
                match repr {
                    Repr::Rust => generate_rust_enum_impl(name, fields.stream()),
                    Repr::C => generate_c_enum_impl(name, fields.stream()),
                }
            }
            other => error_single!(keyword, "Unknown keyword {other:?}"),
        },
        // tuple struct
        (
            Some(TT::Ident(s)),
            Some(TT::Ident(name)),
            Some(TT::Group(fields)),
            Some(TT::Punct(_)),
            None,
        ) => {
            assert_eq!("struct", s.to_string());
            let name = name.to_string();
            let fields = parse_fields(fields.stream())?;
            generate_struct_impl(name, fields)
        }
        other @ _ => {
            panic!("Unsupported struct shape.\n{other:?}")
        }
    }
}

fn generate_struct_impl(name: String, fields: Vec<Field>) -> Result<TokenStream, MacroError> {
    let result = &mut String::new();
    write!(
        result,
        r#"
impl ::quicksilver::Quicksilver for {name} {{
    const MIRROR: ::quicksilver::Type = ::quicksilver::Type::Struct(&::quicksilver::Struct {{
        name: "{name}",
        size: ::std::mem::size_of::<Self>(),
        align: align_of::<Self>(),
        fields: &["#
    )
    .unwrap();

    for (i, field) in fields.into_iter().enumerate() {
        generate_field(
            result,
            &field.name.unwrap_or_else(|| format!("{i}")),
            &field.mirror,
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
::quicksilver::Field {{
    name: "{name}",
    ty: {ty},
    offset: ::std::mem::offset_of!(Self, {name}),
}},"#
    )
    .unwrap()
}

#[derive(Debug)]
struct Field {
    name: Option<String>,
    mirror: String,
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

            let skip = match current {
                Some(TT::Ident(ref ident)) if ident.to_string().as_str() == "pub" => true,
                Some(TT::Group(ref g)) => {
                    let tt = g.stream().into_iter().next();
                    matches!(tt, Some(TT::Ident(c)) if c.to_string().as_str() == "crate")
                }
                _ => false,
            };

            if !skip {
                buffer.push(current.unwrap());
            }
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
            let mirror = parse_mirror(&buffer, &ty.to_string(), skip)?;
            Ok(Field {
                name,
                mirror,
                ty: ty.to_string(),
            })
        }
        (Some(TT::Ident(name)), Some(TT::Punct(colon)), Some(TT::Ident(ty)))
            if colon.as_char() == ':' =>
        {
            let name = Some(name.to_string());
            let buffer = &buffer[2..];
            let mirror = parse_mirror(&buffer, &ty.to_string(), skip)?;
            let ty = buffer
                .iter()
                .map(|it| it.to_string())
                .collect::<Vec<_>>()
                .join("");
            Ok(Field { name, mirror, ty })
        }
        (Some(TT::Ident(ty)), Some(TT::Punct(stair)), Some(_)) if stair.as_char() == '<' => {
            let name = None;
            let mirror = parse_mirror(&buffer, &ty.to_string(), skip)?;
            let ty = buffer
                .iter()
                .map(|it| it.to_string())
                .collect::<Vec<_>>()
                .join("");
            Ok(Field { name, mirror, ty })
        }
        _ => {
            dbg!(&buffer);
            error!(
                &[buffer[0].clone(), buffer.last().unwrap().clone()],
                "Quicksilver can't parse this."
            )
        }
    }
}

fn parse_mirror(buffer: &[TokenTree], ty: &str, skip: bool) -> Result<String, MacroError> {
    Ok(if buffer.len() == 1 {
        if skip {
            error!(
                &[buffer[0].clone(), buffer.last().unwrap().clone()],
                "Quicksilver can't skip this field."
            )
        }
        let mut result = String::new();
        result.push_str(ty);
        result.push_str("::MIRROR");
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
            result.push_str("::MIRROR");
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

fn generate_c_enum_impl(name: String, input: TokenStream) -> Result<TokenStream, MacroError> {
    let result = &mut String::new();
    write!(
        result,
        r#"
impl ::quicksilver::Quicksilver for {name} {{
    const MIRROR: ::quicksilver::Type = ::quicksilver::Type::CEnum(&::quicksilver::CEnum {{
        name: "{name}",
        size: ::std::mem::size_of::<Self>(),
        align: ::std::mem::align_of::<Self>(),
        variants: &["#
    )
    .unwrap();

    let mut i = 0;
    let mut name = String::new();

    for tt in input.into_iter() {
        match tt {
            TT::Ident(ident) => {
                name = ident.to_string();
            }
            TT::Literal(lit) => {
                i = lit.to_string().parse().unwrap_or(i);
            }
            TT::Punct(comma) if comma.as_char() == ',' => {
                write!(result, r#"({i}, "{name}"),"#).unwrap();
                name = String::new();
                i += 1;
            }
            _ => continue,
        }
    }

    if !name.is_empty() {
        write!(result, r#"({i}, "{name}"),"#).unwrap();
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

fn generate_rust_enum_impl(
    enum_name: String,
    input: TokenStream,
) -> Result<TokenStream, MacroError> {
    let result = &mut String::new();
    let variants = parse_rust_enum_variants(input)?;
    let variant_text = &mut String::new();
    for v in &variants {
        let name = &v.name;
        write!(
            variant_text,
            r#"
RustEnumVariant {{
    name: "{name}",
    fields: &["#
        )
        .unwrap();
        for (i, field) in v.fields.iter().enumerate() {
            let name = field.name.clone().unwrap_or_else(|| i.to_string());
            let mirror = &field.mirror;
            write!(variant_text, r#"("{name}", {mirror}),"#).unwrap()
        }
        variant_text.push_str("],}, ");
    }

    let reflect_text = &mut String::new();
    let reflect_ref_text = &mut String::new();

    reflect_ref_text.push_str(r#"todo!()"#);
    reflect_text.push_str(
        r#"let enum_val: &mut Self = unsafe { &mut *(ptr as *mut Self) };
match enum_val {
"#,
    );
    // write match arms
    for (variant_idx, v) in variants.iter().enumerate() {
        // write match arm
        let variant_name = &v.name;
        write!(reflect_text, r#"Self::{variant_name} "#).unwrap();
        // different destructuring depending on tuple struct or "normal" struct
        let is_tuple = v
            .fields
            .first()
            .map(|it| it.name.is_none())
            .unwrap_or(false);
        if is_tuple {
            reflect_text.push('(');
            for (i, _) in v.fields.iter().enumerate() {
                write!(reflect_text, "val{i},").unwrap();
            }
            reflect_text.push(')');
        } else {
            reflect_text.push('{');
            for field in v.fields.iter() {
                let name = field.name.as_ref().unwrap();
                write!(reflect_text, "{name},").unwrap();
            }
            reflect_text.push('}');
        }
        reflect_text.push_str(" => ");

        write!(
            reflect_text,
            r#"
RustEnumReflection {{
                    name: "{enum_name}",
                    variant_name: "{variant_name}",
                    variant_idx: {variant_idx},
                    ty: &Self::MIRROR,
                    fields: vec!["#
        )
        .unwrap();
        for (i, field) in v.fields.iter().enumerate() {
            let name = field.name.clone().unwrap_or_else(|| format!("val{i}"));
            let mirror = &field.mirror;
            write!(reflect_text, r#"
FieldReflection {{
    name: "{name}",
    value: unsafe {{
        reflect_value(&raw mut *{name} as *mut u8, &{mirror})
    }},
}},"#).unwrap();
        }
        reflect_text.push_str("],},");
    }
    reflect_text.push_str("}");

    write!(
        result,
        r#"
impl ::quicksilver::Quicksilver for {enum_name} {{
    const MIRROR: ::quicksilver::Type = ::quicksilver::Type::RustEnum(&::quicksilver::RustEnum {{
        name: "{enum_name}",
        size: ::std::mem::size_of::<Self>(),
        align: ::std::mem::align_of::<Self>(),
        variants: &[{variant_text}],
        reflect: |ptr| {{ {reflect_text} }},
        reflect_ref: |ptr| {{ {reflect_ref_text} }},
        write: |this, variant, fields| {{ todo!() }},
    }});
}}
"#
    )
    .unwrap();

    Ok(result.parse().unwrap())
}

#[derive(Debug)]
struct RustEnumVariant {
    name: String,
    fields: Vec<Field>,
}

fn parse_rust_enum_variants(input: TokenStream) -> Result<Vec<RustEnumVariant>, MacroError> {
    dbg!(&input);
    let mut r = Vec::new();

    let mut iter = input.into_iter();
    loop {
        match (iter.next(), iter.next()) {
            (None, None) => {
                break;
            }
            (Some(TT::Ident(name)), Some(TT::Punct(comma))) if comma.as_char() == ',' => {
                r.push(RustEnumVariant {
                    name: name.to_string(),
                    fields: Vec::new(),
                });
            }
            (Some(TT::Ident(name)), None) => {
                r.push(RustEnumVariant {
                    name: name.to_string(),
                    fields: Vec::new(),
                });
            }
            (Some(TT::Ident(name)), Some(TT::Group(field_group))) => {
                // consume next comma if any
                match iter.next() {
                    None => {}
                    Some(TT::Punct(comma)) if comma.as_char() == ',' => {}
                    Some(unexpected) => {
                        error_single!(&unexpected, "Quicksilver can't parse this.")
                    }
                }
                r.push(RustEnumVariant {
                    name: name.to_string(),
                    fields: parse_fields(field_group.stream())?,
                });
            }
            (Some(other), None) => {
                error_single!(&other, "Quicksilver can't parse this.")
            }

            (Some(other), Some(other2)) => {
                error!(&[other, other2], "Quicksilver can't parse this.")
            }
            (None, Some(_)) => unreachable!(),
        }
    }
    dbg!(&r);
    Ok(r)
}
