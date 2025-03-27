#[cfg(test)]
mod test {
    use diffogus::diff::{Changeable, CollectionDiffEntry, Diffable, HashMapDiff};
    use serde_json::json;

    #[test]
    fn test_serde_changed() {
        let a: i32 = 10;
        let b = 12;
        let diff = a.diff(&b);
        let expected = r#"{"type":"changed","value":{"old":10,"new":12}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_serde_unchanged() {
        let a: f32 = 1.0;
        let b = 1.0 + f32::EPSILON;
        let diff = a.diff(&b);
        let expected = r#"{"type":"unchanged"}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_serde_option() {
        let a: Option<i32> = None;
        let b: Option<i32> = None;
        let diff = a.diff(&b);
        let expected = r#"{"type":"unchanged"}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a: Option<i32> = Some(12);
        let b: Option<i32> = None;
        let diff = a.diff(&b);
        let expected = r#"{"type":"removed","value":12}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a: Option<i32> = None;
        let b: Option<i32> = Some(12);
        let diff = a.diff(&b);
        let expected = r#"{"type":"added","value":12}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a: Option<i32> = Some(12);
        let b: Option<i32> = Some(21);
        let diff = a.diff(&b);
        let expected =
            r#"{"type":"changed","value":{"type":"changed","value":{"old":12,"new":21}}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_vec_serde() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let diff = a.diff(&b);
        let expected = r#"[{"type":"unchanged"},{"type":"unchanged"},{"type":"unchanged"}]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = vec![1, 2, 3];
        let b = vec![2, 2, 3];
        let diff = a.diff(&b);
        let expected = r#"[{"type":"changed","value":{"type":"changed","value":{"old":1,"new":2}}},{"type":"unchanged"},{"type":"unchanged"}]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3, 4];
        let diff = a.diff(&b);
        let expected = r#"[{"type":"unchanged"},{"type":"unchanged"},{"type":"unchanged"},{"type":"added","value":4}]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3];
        let diff = a.diff(&b);
        let expected = r#"[{"type":"unchanged"},{"type":"unchanged"},{"type":"unchanged"},{"type":"removed","value":4}]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_hashmap_serde() {
        let diff_str = json!({
            "0": { "type": "unchanged" },
            "1": { "type": "unchanged" }
        });
        let diff = serde_json::from_value::<HashMapDiff<i32, i32>>(diff_str)
            .unwrap()
            .0;
        assert!(!diff.get(&0).unwrap().is_changed());
        assert!(!diff.get(&0).unwrap().is_changed());

        let diff_str = json!({
           "0": {
               "type": "changed",
               "value": {
                   "type": "changed",
                   "value": { "old": 1, "new": 2 }
               }
           },
           "1": { "type": "unchanged" }
        });
        let diff = serde_json::from_value::<HashMapDiff<i32, i32>>(diff_str)
            .unwrap()
            .0;
        assert!(diff.get(&0).unwrap().is_changed());
        assert!(!diff.get(&1).unwrap().is_changed());

        let diff_str = json!({
           "0": { "type": "unchanged" },
           "1": { "type": "unchanged" },
           "2": {
               "type": "added",
               "value": 3
           }
        });
        let diff = serde_json::from_value::<HashMapDiff<i32, i32>>(diff_str)
            .unwrap()
            .0;
        assert!(!diff.get(&0).unwrap().is_changed());
        assert!(!diff.get(&1).unwrap().is_changed());
        assert_eq!(*diff.get(&2).unwrap(), CollectionDiffEntry::Added(3));

        let diff_str = json!({
           "0": { "type": "unchanged" },
           "1": { "type": "unchanged" },
           "2": {
               "type": "removed",
               "value": 3
           }
        });
        let diff = serde_json::from_value::<HashMapDiff<i32, i32>>(diff_str)
            .unwrap()
            .0;
        assert!(!diff.get(&0).unwrap().is_changed());
        assert!(!diff.get(&1).unwrap().is_changed());
        assert_eq!(*diff.get(&2).unwrap(), CollectionDiffEntry::Removed(3));
    }
}
