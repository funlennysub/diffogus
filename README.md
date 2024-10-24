# diffogus

Rust crate to calculate the difference between 2 instances of the same type

# Features

- Simple diff of basic rust types
    - All integer types
    - All floats
    - Vectors of elements that implement `Diffable`
    - HashMaps where value implements `Diffable`
    - Options of types that implement `Diffable`
- Diff between 2 instances of a struct that implements `Diffable`
    - Implemented manually or with `#[derive(Diff)]`
    - ⚠️ Currently only structs with named fields are supported
- Serialize your diffs with `serde` feature flag