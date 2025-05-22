#[non_exhaustive]
enum Type {
    I32,
    U32,
    F32,
    String,
    Vec(&'static Type),
    Struct(&'static Struct),
}

pub struct Field {
    name: &'static str,
    ty: Type,
    offset: usize,
}

pub struct Struct {
    fields: &'static [Field],
}

pub trait Reflection {
    const MIRROR: &'static Struct;
}

#[repr(C)]
pub enum FieldTypeReflection<'a> {
    I32(&'a mut i32),
    U32(&'a mut u32),
    F32(&'a mut f32),
    String(&'a mut String),
    Struct(&'a mut StructReflection<'a>),
}

#[repr(C)]
pub struct FieldReflection<'a> {
    pub name: &'static str,
    pub ty: FieldTypeReflection<'a>,
}

#[repr(C)]
pub struct StructReflection<'a> {
    pub fields: Vec<FieldReflection<'a>>,
}

pub fn reflect<T: Reflection>(val: &mut T) -> StructReflection<'_> {
    let mut fields: Vec<FieldReflection> = Vec::new();
    for field in T::MIRROR.fields {
        match field.ty {
            Type::I32 => {
                let value = unsafe {
                    let ptr = (val as *mut T as *mut u8).add(field.offset) as *mut i32;
                    &mut *ptr
                };
                fields.push(FieldReflection {
                    name: field.name,
                    ty: FieldTypeReflection::I32(value),
                });
            }
            _ => {
                todo!();
            }
        }
    }
    StructReflection { fields }
}


