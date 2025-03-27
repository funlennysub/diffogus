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
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

/// Represents the difference between two [`Map`] collections.
#[derive(Debug, Serialize, Deserialize)]
pub struct ValueMapDiff(pub BTreeMap<String, CollectionDiffEntry<Value>>);

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
        let mut out = BTreeMap::new();

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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
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
