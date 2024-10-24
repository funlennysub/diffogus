#[cfg(test)]
mod tests {
    use diffogus::diff::{Changeable, CollectionDiffEntry, Diffable, OptionDiff, PrimitiveDiff};
    use std::collections::HashMap;

    #[test]
    fn test_primitive_diff() {
        // Integer diff
        let a: i32 = 5;
        let b = 10;
        let diff = a.diff(&b);
        assert!(diff.is_changed());
        if let PrimitiveDiff::Changed { old, new } = diff {
            assert_eq!(old, 5);
            assert_eq!(new, 10);
        }

        // Float diff (with EPSILON)
        let a = 1.0;
        let b = 1.0 + f64::EPSILON;
        let diff = a.diff(&b);
        assert!(!diff.is_changed()); // No significant change

        // Boolean diff
        let a = true;
        let b = false;
        let diff = a.diff(&b);
        assert!(diff.is_changed());
    }

    #[test]
    fn test_string_diff() {
        let a = String::from("hello");
        let b = String::from("world");
        let diff = a.diff(&b);

        assert!(diff.is_changed());
        if let PrimitiveDiff::Changed { old, new } = diff {
            assert_eq!(old, "hello");
            assert_eq!(new, "world");
        }

        // Unchanged case
        let a = String::from("same");
        let b = String::from("same");
        let diff = a.diff(&b);
        assert!(!diff.is_changed());
    }

    #[test]
    fn test_option_diff() {
        let a: Option<i32> = Some(10);
        let b: Option<i32> = Some(20);

        let diff = a.diff(&b);
        assert!(diff.is_changed());
        if let OptionDiff::Changed(PrimitiveDiff::Changed { old, new }) = diff {
            assert_eq!(old, 10);
            assert_eq!(new, 20);
        }

        // Removed case
        let a: Option<i32> = Some(10);
        let b: Option<i32> = None;
        let diff = a.diff(&b);
        assert!(diff.is_changed());
        if let OptionDiff::Removed(value) = diff {
            assert_eq!(value, 10);
        }

        // Unchanged case
        let a: Option<i32> = None;
        let b: Option<i32> = None;
        let diff = a.diff(&b);
        assert!(!diff.is_changed());
    }

    #[test]
    fn test_vec_diff() {
        let a = vec![1, 2, 3];
        let b = vec![1, 4, 3];

        let diff = a.diff(&b);
        assert!(diff.is_changed());

        let vec = diff.0;
        assert!(matches!(vec[0], CollectionDiffEntry::Unchanged));
        assert!(matches!(vec[1], CollectionDiffEntry::Changed(_)));
        assert!(matches!(vec[2], CollectionDiffEntry::Unchanged));

        // Unchanged case
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let diff = a.diff(&b);
        assert!(!diff.is_changed());
    }

    #[test]
    fn test_hashmap_diff() {
        let mut a = HashMap::new();
        a.insert("key1", 1);
        a.insert("key2", 2);

        let mut b = HashMap::new();
        b.insert("key1", 1); // unchanged
        b.insert("key2", 3); // changed
        b.insert("key3", 4); // added

        let diff = a.diff(&b);
        assert!(diff.is_changed());

        let map = diff.0;
        assert!(matches!(map["key1"], CollectionDiffEntry::Unchanged));
        assert!(matches!(map["key2"], CollectionDiffEntry::Changed(_)));
        assert!(matches!(map["key3"], CollectionDiffEntry::Added(_)));

        // Unchanged case
        let diff = a.diff(&a);
        assert!(!diff.is_changed());
    }
}
