#![deny(missing_docs)]

//! # diffogus
//!
//! This crate provides a flexible and customizable framework for computing differences ("diffs")
//! between various types of data structures in Rust. The core functionality is built around the
//! traits [`diff::Diffable`] and [`diff::Changeable`], which allow you to determine changes between two instances
//! of a type and represent those changes in a structured way.
//!
//! ## Overview
//!
//! The crate supports diffing primitive types (e.g., integers, floats, strings), as well as
//! collections like `HashMap`, `Vec`, and `Option`. It can be extended to handle custom types
//! by implementing the [`diff::Diffable`] trait.
//!
//! The result of a `diff` operation can be serialized into formats such as JSON, provided the
//! `serde` feature is enabled.
//!
//! ## Traits
//!
//! - [`diff::Changeable`] - A trait for types that can report whether they have changed.
//! - [`diff::Diffable`] - A trait for types that can compute a difference with another instance of the same type.
//!
//! ## Supported Types
//!
//! By default, this crate implements [`diff::Diffable`] for most types.
//!
//! Full list of types:
//!
//! - Primitive types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool`, and `String`.
//! - Collections: `HashMap<K, V>`, `Vec<T>`.
//! - Containers: `Option<T>`.
//!
//! ## Features
//!
//! - **`serde`**: Enables support for serializing diff results using `serde`.
//! - **`derive`**: Enables support for [`Diff`] derive macro.
//!
//! ## Usage
//!
//! ### Basic Diffing
//!
//! Here's a simple example that shows how to compute a diff between two integers:
//!
//! ```rust
//! use diffogus::diff::{Diffable, PrimitiveDiff};
//!
//! let a = 5;
//! let b = 10;
//! let diff = a.diff(&b);
//!
//! match diff {
//!     PrimitiveDiff::Changed { old, new } => {
//!         println!("Value changed from {} to {}", old, new);
//!     },
//!     PrimitiveDiff::Unchanged => {
//!         println!("No change detected.");
//!     }
//! }
//! ```
//!
//! ### Diffing more complex data structures
//!
//! You can also compute diffs on more complex structures such as `HashMap` or `Vec`.
//!
//! ```rust
//! use diffogus::diff::{Diffable, HashMapDiff, CollectionDiffEntry};
//! use std::collections::HashMap;
//!
//! let mut map1 = HashMap::new();
//! map1.insert("key1".to_string(), 1);
//! map1.insert("key2".to_string(), 2);
//!
//! let mut map2 = HashMap::new();
//! map2.insert("key1".to_string(), 1);
//! map2.insert("key2".to_string(), 3);
//! map2.insert("key3".to_string(), 4);
//!
//! let diff = map1.diff(&map2);
//!
//! for (key, diff_entry) in diff.0 {
//!     match diff_entry {
//!         CollectionDiffEntry::Added(new_val) => println!("{} was added with value {}", key, new_val),
//!         CollectionDiffEntry::Removed(old_val) => println!("{} was removed with value {}", key, old_val),
//!         CollectionDiffEntry::Changed(changed_val) => println!("{} changed", key),
//!         CollectionDiffEntry::Unchanged => println!("{} did not change", key),
//!     }
//! }
//! ```
//!
//! ### Serde Integration
//!
//! If you want to serialize the diff result (e.g., to JSON), enable the `serde` feature:
//!
//! ```toml
//! [dependencies]
//! diffogus = { version = "0.1", features = ["serde"] }
//! ```
//!
//! Example usage:
//!
//! ```no_run
//! use diffogus::diff::{Diffable, HashMapDiff};
//! use serde_json;
//! use std::collections::HashMap;
//!
//! let mut map1 = HashMap::new();
//! map1.insert("key1".to_string(), 1);
//!
//! let mut map2 = HashMap::new();
//! map2.insert("key1".to_string(), 2);
//!
//! let diff = map1.diff(&map2);
//! let serialized = serde_json::to_string(&diff).unwrap();
//!
//! println!("Serialized diff: {}", serialized);
//! ```
//!
//! ### Derive macro
//!
//! If you want to implement [`diff::Diffable`] and related traits you can do that very easily by enabling `derive` feature:
//!
//! ```toml
//! [dependencies]
//! diffogus = { version = "0.1", features = ["derive"] }
//! ```
//!
//! Example usage:
//!
//! ```no_run
//! use diffogus::diff::{Diffable, PrimitiveDiff};
//! use diffogus_derive::Diff;
//!
//! #[derive(Debug, Clone, Diff)]
//! struct MyStruct {
//!     id: u32,
//!     name: String,
//! }
//!
//! let a = MyStruct { id: 0, name: "Joe".into() };
//! let b = MyStruct { id: 0, name: "Doe".into() };
//!
//! let diff = a.diff(&b);
//! // Now do whatever you want with this diff.
//! ```
//!

#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]

/// Custom trait to allow for conditional serde support
#[cfg(feature = "serde")]
pub trait MySerialize<'de>: Serialize + Deserialize<'de> {}

/// Custom trait to allow for conditional serde support
#[cfg(not(feature = "serde"))]
pub trait MySerialize<'de> {}

#[cfg(feature = "serde")]
impl<'de, T> MySerialize<'de> for T where T: Serialize + Deserialize<'de> {}

#[cfg(not(feature = "serde"))]
impl<T> MySerialize<'_> for T {}

/// Core diffing implementation
pub mod diff;

/// Diffing implementation for `serde_json::Value`
#[cfg(feature = "json_value")]
#[cfg_attr(docsrs, doc(cfg(feature = "json_value")))]
pub mod json_value;

#[cfg(feature = "diffogus_derive")]
extern crate diffogus_derive;

#[cfg(feature = "diffogus_derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use diffogus_derive::Diff;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
