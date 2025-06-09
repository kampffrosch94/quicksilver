# quicksilver

`quicksilver` is to `facet` what `nanoserde` is to `serde`

It is limited in scope and features. 
It doesn't have external dependencies.
It has minimal compile time impact.

## Demo

TODO

## Use cases
- serialization/deserialization roundtrip 
- Adhoc UI Editors (see example folder)

## Supported Types

| Type                  | Status |
|-----------------------|--------|
| `u32`, `i32`, `f32`   | ✅     |
| `u64`, `i64`,  `f64`  | ✅     |
| `usize`, `isize`      | ✅     |
| `bool`                | ✅     |
| `String`              | ✅     |
| `Vec<T>`              | ✅     |
| `HashMap<K,V>`        | ✅     |
| `Option<T>`           | ✅     |
| custom `struct`       | ✅     |
| custom C-Style `enum` | ✅     |
| `Box<T>`              | ⛔     |
| regular Rust `enum`   | ⛔     |

Quicksilver can be derived for structs and `repr(C)` enums via `#[derive(Quicksilver)]`.

If a container contains an unsupported type it can be skipped with the attribute `#[quicksilver(skip)]`.


## Limitations

- The json serializer/deserializer is not general purpose. It is only useable for roundtrips. It can't read arbitrary data. It just panics on error. Good enough for me, but maybe not for you.
- Since quicksilver builds on `const` cycles are not supported. You can't store a `T` inside a `T`, even transitively.
- Adding elements to a collection via the inspector is intended, but not yet fleshed out.
