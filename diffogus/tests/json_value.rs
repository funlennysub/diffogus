#[cfg(test)]
mod test {
    use diffogus::diff::*;
    use diffogus::json_value::*;
    use serde_json::{json, Number};

    #[test]
    fn test_value_diff_basic() {
        let a = json!(null);
        let b = json!(null);
        let diff = a.diff(&b);
        assert!(!diff.is_changed());

        let a = json!("Hello World!");
        let b = json!("Hello World");
        let diff = a.diff(&b);
        assert_eq!(
            ValueDiff::StringChanged {
                old: "Hello World!".into(),
                new: "Hello World".into()
            },
            diff
        );

        let a = json!(false);
        let b = json!(true);
        let diff = a.diff(&b);
        assert_eq!(
            ValueDiff::BoolChanged {
                old: false,
                new: true
            },
            diff
        );

        let a = json!(10.0);
        let b = json!(5.0);
        let diff = a.diff(&b);
        assert_eq!(
            ValueDiff::NumberChanged {
                old: Number::from_f64(10.0).unwrap(),
                new: Number::from_f64(5.0).unwrap()
            },
            diff
        );
    }

    #[test]
    fn test_value_diff_more() {
        let a = json!({
            "size": 10,
            "name": "pen"
        });
        let b = json!({
            "size": 11,
            "name": "pen"
        });
        let diff = a.diff(&b);
        if let ValueDiff::ObjectChanged(obj) = diff {
            let obj = obj.0;
            assert!(matches!(obj["size"], CollectionDiffEntry::Changed(_)));
            assert!(matches!(obj["name"], CollectionDiffEntry::Unchanged));
        } else {
            unreachable!("Object diff is not `ObjectChanged`")
        }

        let a = json!([1, 2, "Hello"]);
        let b = json!(["hello", 2, 3]);
        let diff = a.diff(&b);
        if let ValueDiff::ArrayChanged(array) = diff {
            let array = array.0;
            assert!(matches!(
                array[0],
                CollectionDiffEntry::Changed(ValueDiff::VariantChanged { .. })
            ));
            assert!(matches!(array[1], CollectionDiffEntry::Unchanged));
            assert!(matches!(
                array[2],
                CollectionDiffEntry::Changed(ValueDiff::VariantChanged { .. })
            ));
        } else {
            unreachable!("Array diff is not `ArrayChanged`")
        }
    }

    #[test]
    fn test_value_diff_serde() {
        let a = json!(null);
        let b = json!(null);
        let diff = a.diff(&b);
        let expected = r#"{"type":"unchanged"}"#;
        assert_eq!(expected, serde_json::to_string(&diff).unwrap());

        let a = json!("Hello World!");
        let b = json!("Hello World");
        let diff = a.diff(&b);
        let expected =
            r#"{"type":"string_changed","value":{"old":"Hello World!","new":"Hello World"}}"#;
        assert_eq!(expected, serde_json::to_string(&diff).unwrap());

        let a = json!({
            "size": 10,
            "name": "box"
        });
        let b = json!({
            "size": 11,
            "name": "box"
        });
        let diff = a.diff(&b);
        let expected = r#"{"type":"object_changed","value":{"name":{"type":"unchanged"},"size":{"type":"changed","value":{"type":"number_changed","value":{"old":10,"new":11}}}}}"#;
        assert_eq!(expected, serde_json::to_string(&diff).unwrap());

        let a = json!({
            "arr": [1, 2, 3]
        });
        let b = json!({
            "arr": [2, 2, 3]
        });
        let diff = a.diff(&b);
        let expected = r#"{"type":"object_changed","value":{"arr":{"type":"changed","value":{"type":"array_changed","value":[{"type":"changed","value":{"type":"number_changed","value":{"old":1,"new":2}}},{"type":"unchanged"},{"type":"unchanged"}]}}}}"#;
        assert_eq!(expected, serde_json::to_string(&diff).unwrap());

        let a = json!({
            "nested": {
                "deep": true
            }
        });
        let b = json!({
            "nested": {
                "deep": "very"
            }
        });
        let diff = a.diff(&b);
        let expected = r#"{"type":"object_changed","value":{"nested":{"type":"changed","value":{"type":"object_changed","value":{"deep":{"type":"changed","value":{"type":"variant_changed","value":{"old":true,"new":"very"}}}}}}}}"#;
        assert_eq!(expected, serde_json::to_string(&diff).unwrap());
    }
}
