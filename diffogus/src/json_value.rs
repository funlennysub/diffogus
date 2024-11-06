//! # Diffing implementation for [`serde_json::Value`]
//!
//! ```no_run
//! use serde_json::json;
//! use diffogus::{diff::Diffable, json_value::ValueDiff};
//!
//! let a = json!(null);
//! let b = json!(true);
//! let diff = a.diff(&b);
//! assert_eq!(ValueDiff::VariantChanged { old: a, new: b }, diff);
//!```
//!

use crate::diff::{Changeable, CollectionDiffEntry, Diffable, PrimitiveDiff, VecDiff};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};
use serde_json::{Map, Number, Value};
use std::collections::HashMap;

/// Represents the difference between two [`Map`] collections.
#[derive(Debug)]
pub struct ValueMapDiff(pub HashMap<String, CollectionDiffEntry<Value>>);

#[cfg(feature = "serde")]
impl Serialize for ValueMapDiff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.0.values().filter(|v| v.is_changed()).count();
        let mut map = serializer.serialize_map(Some(len))?;

        for (k, v) in self.0.iter().filter(|(_, v)| v.is_changed()) {
            map.serialize_entry(k, v)?;
        }

        map.end()
    }
}

impl PartialEq for ValueMapDiff {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Changeable for ValueMapDiff {
    fn is_changed(&self) -> bool {
        self.0.values().any(|v| v.is_changed())
    }
}

impl Diffable for Map<String, Value> {
    type Repr = ValueMapDiff;

    fn diff(&self, b: &Self) -> Self::Repr {
        let mut out = HashMap::new();

        for (k, v) in self {
            let other = b.get(k);
            match other {
                Some(other) => {
                    let diff = v.diff(other);
                    if diff.is_changed() {
                        out.insert(k.clone(), CollectionDiffEntry::Changed(diff))
                    } else {
                        out.insert(k.clone(), CollectionDiffEntry::Unchanged)
                    }
                }
                None => out.insert(k.clone(), CollectionDiffEntry::Removed(v.clone())),
            };
        }

        for (k, v) in b {
            if out.contains_key(k) {
                continue;
            }
            out.insert(k.clone(), CollectionDiffEntry::Added(v.clone()));
        }

        ValueMapDiff(out)
    }
}

/// Enum representing a difference between two [`Value`]
#[derive(Debug)]
pub enum ValueDiff {
    /// Indicates that the value has not changed.
    Unchanged,
    /// Indicated that the enum variant has changed.
    VariantChanged {
        /// Field holding the old value
        old: Value,
        /// Field holding the new value
        new: Value,
    },
    /// Indicated that [`Value::Bool`] value has changed.
    BoolChanged {
        /// Field holding the old value
        old: bool,
        /// Field holding the new value
        new: bool,
    },
    /// Indicated that [`Value::String`] value has changed.
    StringChanged {
        /// Field holding the old value
        old: String,
        /// Field holding the new value
        new: String,
    },
    /// Indicated that [`Value::Number`] value has changed.
    NumberChanged {
        /// Field holding the old value
        old: Number,
        /// Field holding the new value
        new: Number,
    },
    /// Indicates that [`Value::Array`] values have changed.
    ArrayChanged(VecDiff<Value>),
    /// Indicates that [`Value::Object`] values have changed.
    ObjectChanged(ValueMapDiff),
}

#[cfg(feature = "serde")]
impl Serialize for ValueDiff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueDiff::Unchanged => serializer.serialize_unit(),
            ValueDiff::VariantChanged { old, new } => {
                let mut state = serializer.serialize_struct("ValueDiff", 3)?;

                state.serialize_field("old", old)?;
                state.serialize_field("new", new)?;
                state.serialize_field("type", "variant_changed")?;

                state.end()
            }
            ValueDiff::BoolChanged { old, new } => {
                let mut state = serializer.serialize_struct("ValueDiff", 3)?;

                state.serialize_field("old", old)?;
                state.serialize_field("new", new)?;
                state.serialize_field("type", "bool_changed")?;

                state.end()
            }
            ValueDiff::StringChanged { old, new } => {
                let mut state = serializer.serialize_struct("ValueDiff", 3)?;

                state.serialize_field("old", old)?;
                state.serialize_field("new", new)?;
                state.serialize_field("type", "string_changed")?;

                state.end()
            }
            ValueDiff::NumberChanged { old, new } => {
                let mut state = serializer.serialize_struct("ValueDiff", 3)?;

                state.serialize_field("old", old)?;
                state.serialize_field("new", new)?;
                state.serialize_field("type", "number_changed")?;

                state.end()
            }
            ValueDiff::ArrayChanged(diff) => {
                let mut state = serializer.serialize_seq(Some(diff.0.len()))?;

                for e in &diff.0 {
                    state.serialize_element(e)?;
                }

                state.end()
            }
            ValueDiff::ObjectChanged(diff) => {
                let len = diff.0.values().filter(|v| v.is_changed()).count();
                let mut map = serializer.serialize_map(Some(len))?;

                for (k, v) in diff.0.iter().filter(|(_, v)| v.is_changed()) {
                    map.serialize_entry(k, v)?;
                }

                map.end()
            }
        }
    }
}

impl PartialEq for ValueDiff {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::VariantChanged {
                    old: a_old,
                    new: a_new,
                },
                Self::VariantChanged {
                    old: b_old,
                    new: b_new,
                },
            ) => a_old == b_old && a_new == b_new,
            (
                Self::BoolChanged {
                    old: a_old,
                    new: a_new,
                },
                Self::BoolChanged {
                    old: b_old,
                    new: b_new,
                },
            ) => a_old == b_old && a_new == b_new,
            (
                Self::StringChanged {
                    old: a_old,
                    new: a_new,
                },
                Self::StringChanged {
                    old: b_old,
                    new: b_new,
                },
            ) => a_old == b_old && a_new == b_new,
            (
                Self::NumberChanged {
                    old: a_old,
                    new: a_new,
                },
                Self::NumberChanged {
                    old: b_old,
                    new: b_new,
                },
            ) => a_old == b_old && a_new == b_new,
            (Self::ArrayChanged(a), Self::ArrayChanged(b)) => a == b,
            (Self::ObjectChanged(a), Self::ObjectChanged(b)) => a == b,
            _ => false,
        }
    }
}

impl Changeable for ValueDiff {
    fn is_changed(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

impl Diffable for Value {
    type Repr = ValueDiff;

    fn diff(&self, b: &Self) -> Self::Repr {
        match (self, b) {
            (Self::Null, Self::Null) => ValueDiff::Unchanged,
            (Self::Bool(a), Self::Bool(b)) => match a.diff(b) {
                PrimitiveDiff::Changed { old, new } => ValueDiff::BoolChanged { old, new },
                PrimitiveDiff::Unchanged => ValueDiff::Unchanged,
            },
            (Self::Number(na), Self::Number(nb)) => match na == nb {
                true => ValueDiff::Unchanged,
                false => ValueDiff::NumberChanged {
                    old: na.clone(),
                    new: nb.clone(),
                },
            },
            (Self::String(a), Self::String(b)) => match a.diff(b) {
                PrimitiveDiff::Changed { old, new } => ValueDiff::StringChanged { old, new },
                PrimitiveDiff::Unchanged => ValueDiff::Unchanged,
            },
            (Self::Array(a), Self::Array(b)) => {
                let diff = a.diff(b);
                match diff.is_changed() {
                    true => ValueDiff::ArrayChanged(diff),
                    false => ValueDiff::Unchanged,
                }
            }
            (Self::Object(a), Self::Object(b)) => {
                let diff = a.diff(b);
                match diff.is_changed() {
                    true => ValueDiff::ObjectChanged(diff),
                    false => ValueDiff::Unchanged,
                }
            }
            _ => ValueDiff::VariantChanged {
                old: self.clone(),
                new: b.clone(),
            },
        }
    }
}
