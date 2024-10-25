#[cfg(feature = "serde")]
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant},
    Serialize, Serializer,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Trait representing an object that can determine if it has changed.
pub trait Changeable {
    /// Checks if the object has changed.
    fn is_changed(&self) -> bool;

    /// Static method to check if an object has not changed.
    fn is_unchanged(obj: &Self) -> bool {
        !obj.is_changed()
    }
}

/// Trait representing the ability to compute a difference between two objects.
pub trait Diffable {
    /// The type used to represent the difference between two objects.
    type Repr: Changeable + Debug;

    /// Computes the difference between `self` and another object of the same type.
    fn diff(&self, b: &Self) -> Self::Repr;
}

/// Enum representing the difference between two primitive values.
#[derive(Debug)]
pub enum PrimitiveDiff<T: Diffable> {
    /// Indicates that the value has changed, storing the old and new values.
    Changed {
        /// Field holding the old value.
        old: T,
        /// Field holding the new value.
        new: T,
    },
    /// Indicates that the value has not changed.
    Unchanged,
}

impl<T> PartialEq for PrimitiveDiff<T>
where
    T: Diffable + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Changed {
                    old: l_old,
                    new: l_new,
                },
                Self::Changed {
                    old: r_old,
                    new: r_new,
                },
            ) => l_old == r_old && l_new == r_new,
            (Self::Unchanged, Self::Unchanged) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for PrimitiveDiff<T>
where
    T: Diffable + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PrimitiveDiff::Changed { old, new } => {
                let mut state = serializer.serialize_struct("PrimitiveDiff", 3)?;

                state.serialize_field("old", old)?;
                state.serialize_field("new", new)?;
                state.serialize_field("type", "changed")?;

                state.end()
            }
            PrimitiveDiff::Unchanged => serializer.serialize_unit(),
        }
    }
}

impl<T: Diffable> Changeable for PrimitiveDiff<T> {
    fn is_changed(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

/// Macro to implement the `Diffable` trait for integer types.
#[doc(hidden)]
macro_rules! impl_ints {
    ($ty:ty) => {
        impl Diffable for $ty {
            type Repr = PrimitiveDiff<$ty>;

            fn diff(&self, b: &Self) -> Self::Repr {
                if self == b {
                    PrimitiveDiff::Unchanged
                } else {
                    PrimitiveDiff::Changed { old: *self, new: *b }
                }
            }
        }
    };
    ($($ty:ty),*) => {
        $(impl_ints!($ty);)*
    };
}

impl_ints!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool);

/// Macro to implement the `Diffable` trait for floating point types.
#[doc(hidden)]
macro_rules! impl_floats {
    ($ty:ty) => {
        impl Diffable for $ty {
            type Repr = PrimitiveDiff<$ty>;

            fn diff(&self, b: &Self) -> Self::Repr {
                if (b - self).abs() <= <$ty>::EPSILON {
                    PrimitiveDiff::Unchanged
                } else {
                    PrimitiveDiff::Changed { old: *self, new: *b }
                }
            }
        }
    };
    ($($ty:ty),*) => {
        $(impl_floats!($ty);)*
    };
}

impl_floats!(f32, f64);

impl Diffable for String {
    type Repr = PrimitiveDiff<String>;

    fn diff(&self, b: &Self) -> Self::Repr {
        if self == b {
            PrimitiveDiff::Unchanged
        } else {
            PrimitiveDiff::Changed {
                old: self.clone(),
                new: b.clone(),
            }
        }
    }
}

/// Enum representing a difference in collections such as `HashMap` or `Vec`.
#[derive(Debug)]
pub enum CollectionDiffEntry<T: Diffable> {
    /// Indicates that an item was removed from the collection.
    Removed(T),
    /// Indicates that an item was added to the collection.
    Added(T),
    /// Indicates that an item has changed.
    Changed(<T as Diffable>::Repr),
    /// Indicates that an item has not changed.
    Unchanged,
}

impl<T> PartialEq for CollectionDiffEntry<T>
where
    T: Diffable + PartialEq,
    <T as Diffable>::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Removed(l), Self::Removed(r)) => l == r,
            (Self::Added(l), Self::Added(r)) => l == r,
            (Self::Changed(l), Self::Changed(r)) => l == r,
            (Self::Unchanged, Self::Unchanged) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for CollectionDiffEntry<T>
where
    T: Diffable + Serialize,
    <T as Diffable>::Repr: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CollectionDiffEntry::Removed(a) => {
                let mut state = serializer.serialize_struct("Removed", 2)?;
                state.serialize_field("old", a)?;
                state.serialize_field("type", "removed")?;
                state.end()
            }
            CollectionDiffEntry::Added(a) => {
                let mut state = serializer.serialize_struct("Added", 2)?;
                state.serialize_field("new", a)?;
                state.serialize_field("type", "added")?;
                state.end()
            }
            CollectionDiffEntry::Changed(a) => a.serialize(serializer),
            CollectionDiffEntry::Unchanged => serializer.serialize_unit(),
        }
    }
}

impl<T: Diffable> Changeable for CollectionDiffEntry<T> {
    fn is_changed(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

/// Represents the difference between two `HashMap` collections.
#[derive(Debug)]
pub struct HashMapDiff<K, T>(pub HashMap<K, CollectionDiffEntry<T>>)
where
    K: Hash + Eq,
    T: Diffable;

#[cfg(feature = "serde")]
impl<K, T> Serialize for HashMapDiff<K, T>
where
    K: Hash + Eq + Serialize,
    T: Diffable + Serialize,
    <T as Diffable>::Repr: Serialize,
{
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

impl<K, T> Changeable for HashMapDiff<K, T>
where
    K: Hash + Eq,
    T: Diffable,
{
    fn is_changed(&self) -> bool {
        self.0.values().any(|v| v.is_changed())
    }
}

impl<K, T> Diffable for HashMap<K, T>
where
    K: Hash + Eq + Debug + Clone,
    T: Diffable + Debug + Clone,
{
    type Repr = HashMapDiff<K, T>;

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

        HashMapDiff(out)
    }
}

/// Represents the difference between two `Vec` collections.
#[derive(Debug)]
pub struct VecDiff<T: Diffable>(pub Vec<CollectionDiffEntry<T>>);

#[cfg(feature = "serde")]
impl<T> Serialize for VecDiff<T>
where
    T: Diffable + Serialize,
    <T as Diffable>::Repr: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        for e in &self.0 {
            seq.serialize_element(e)?;
        }

        seq.end()
    }
}

impl<T: Diffable> Changeable for VecDiff<T> {
    fn is_changed(&self) -> bool {
        self.0.iter().any(|d| d.is_changed())
    }
}

impl<T> Diffable for Vec<T>
where
    T: Diffable + Debug + Clone,
{
    type Repr = VecDiff<T>;

    fn diff(&self, b: &Self) -> Self::Repr {
        let mut out = vec![];

        let len = self.len().max(b.len());

        for i in 0..len {
            let old = self.get(i);
            let new = b.get(i);

            match (old, new) {
                (Some(a), None) => out.push(CollectionDiffEntry::Removed(a.clone())),
                (Some(a), Some(b)) => {
                    let diff = a.diff(b);
                    if diff.is_changed() {
                        out.push(CollectionDiffEntry::Changed(diff))
                    } else {
                        out.push(CollectionDiffEntry::Unchanged)
                    }
                }
                (None, None) => out.push(CollectionDiffEntry::Unchanged),
                (None, Some(b)) => out.push(CollectionDiffEntry::Added(b.clone())),
            }
        }

        VecDiff(out)
    }
}

/// Enum representing the difference between two `Option` values.
#[derive(Debug)]
pub enum OptionDiff<T: Diffable> {
    /// Indicates that a value was removed (i.e., `Some` became `None`).
    Removed(T),
    /// Indicates that a value was added (i.e., `None` became `Some`).
    Added(T),
    /// Indicates that the inner value of `Some` has changed.
    Changed(<T as Diffable>::Repr),
    /// Indicates that the value has not changed.
    Unchanged,
}

impl<T> PartialEq for OptionDiff<T>
where
    T: Diffable + PartialEq,
    <T as Diffable>::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Removed(l), Self::Removed(r)) => l == r,
            (Self::Added(l), Self::Added(r)) => l == r,
            (Self::Changed(l), Self::Changed(r)) => l == r,
            (Self::Unchanged, Self::Unchanged) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for OptionDiff<T>
where
    T: Diffable + Serialize,
    <T as Diffable>::Repr: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OptionDiff::Removed(a) => {
                let mut state = serializer.serialize_struct_variant("Removed", 0, "removed", 1)?;
                state.serialize_field("old", a)?;
                state.end()
            }
            OptionDiff::Added(a) => {
                let mut state = serializer.serialize_struct_variant("Added", 0, "added", 1)?;
                state.serialize_field("new", a)?;
                state.end()
            }
            OptionDiff::Changed(a) => a.serialize(serializer),
            OptionDiff::Unchanged => serializer.serialize_unit(),
        }
    }
}

impl<T: Diffable> Changeable for OptionDiff<T> {
    fn is_changed(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

impl<T> Diffable for Option<T>
where
    T: Diffable + Clone + Debug,
{
    type Repr = OptionDiff<T>;

    fn diff(&self, b: &Self) -> Self::Repr {
        match (self, b) {
            (Some(a), Some(b)) => {
                let diffed = a.diff(b);
                if diffed.is_changed() {
                    OptionDiff::Changed(diffed)
                } else {
                    OptionDiff::Unchanged
                }
            }
            (Some(a), None) => OptionDiff::Removed(a.clone()),
            (None, Some(a)) => OptionDiff::Added(a.clone()),
            (None, None) => OptionDiff::Unchanged,
        }
    }
}
