# quicksilver

`quicksilver` is to `facet` what `nanoserde` is to `serde`

It is limited in scope and features. 
It doesn't have external dependencies.
It has minimal compile time impact.

## Use cases
- serialization/deserialization roundtrip
- Adhoc UI Editors

## Supported Types

| Type    | Status |
| -------- | ------- |
| `u32`      | :white_check_mark:   |
| `i32`      | :white_check_mark:   |
| `f32`      | :white_check_mark:   |
| `u64`      | 🚧   |
| `i64`      | 🚧   |
| `usize`      | 🚧   |
| `isize`      | 🚧   |
| `f64`      | 🚧   |
| `bool`     | :white_check_mark:   |
| `String`   | :white_check_mark:   |
| `Vec<T>`      | :white_check_mark:   |
| `HashMap<K,V>`      | 🚧   |
| `Option<T>`      | 🚧   |
| custom `struct`      | ✅   |
| custom C-Style `enum`      | 🚧   |


