#[cfg(test)]
mod test {
    use diffogus::diff::Diffable;
    use std::collections::HashMap;

    macro_rules! hashmap {
        ($(($k:expr,$v:expr)),*) => {
            HashMap::from([$(($k,$v)),*])
        };
    }

    #[test]
    fn test_serde_changed() {
        let a: i32 = 10;
        let b = 12;
        let diff = a.diff(&b);
        let expected = r#"{"old":10,"new":12,"type":"changed"}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_serde_unchanged() {
        let a: f32 = 1.0;
        let b = 1.0 + f32::EPSILON;
        let diff = a.diff(&b);
        let expected = r#"null"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_serde_option() {
        let a: Option<i32> = None;
        let b: Option<i32> = None;
        let diff = a.diff(&b);
        let expected = r#"null"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a: Option<i32> = Some(12);
        let b: Option<i32> = None;
        let diff = a.diff(&b);
        let expected = r#"{"old":12,"type":"removed"}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a: Option<i32> = None;
        let b: Option<i32> = Some(12);
        let diff = a.diff(&b);
        let expected = r#"{"new":12,"type":"added"}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a: Option<i32> = Some(12);
        let b: Option<i32> = Some(21);
        let diff = a.diff(&b);
        let expected = r#"{"old":12,"new":21,"type":"changed"}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_vec_serde() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let diff = a.diff(&b);
        let expected = r#"[null,null,null]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = vec![1, 2, 3];
        let b = vec![2, 2, 3];
        let diff = a.diff(&b);
        let expected = r#"[{"old":1,"new":2,"type":"changed"},null,null]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3, 4];
        let diff = a.diff(&b);
        let expected = r#"[null,null,null,{"new":4,"type":"added"}]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3];
        let diff = a.diff(&b);
        let expected = r#"[null,null,null,{"old":4,"type":"removed"}]"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_hashmap_serde() {
        let a = hashmap![(0, 1), (1, 2)];
        let b = hashmap![(0, 1), (1, 2)];
        let diff = a.diff(&b);
        let expected = r#"{}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = hashmap![(0, 1), (1, 2)];
        let b = hashmap![(0, 2), (1, 2)];
        let diff = a.diff(&b);
        let expected = r#"{"0":{"old":1,"new":2,"type":"changed"}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = hashmap![(0, 1), (1, 2)];
        let b = hashmap![(0, 1), (1, 2), (2, 3)];
        let diff = a.diff(&b);
        let expected = r#"{"2":{"new":3,"type":"added"}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = hashmap![(0, 1), (1, 2), (2, 3)];
        let b = hashmap![(0, 1), (1, 2)];
        let diff = a.diff(&b);
        let expected = r#"{"2":{"old":3,"type":"removed"}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }
}
