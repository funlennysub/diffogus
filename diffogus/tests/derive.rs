#[cfg(test)]
mod test {
    use diffogus::diff::{Diffable, PrimitiveDiff};
    use diffogus::Diff;

    #[test]
    fn test_derive() {
        #[derive(Debug, Diff)]
        struct Ball {
            size: f32,
            color: String,
        }

        let a = Ball {
            size: 10.0,
            color: "Red".into(),
        };
        let b = Ball {
            size: 23.0,
            color: "Red".into(),
        };
        let diff = a.diff(&b);
        assert_eq!(
            PrimitiveDiff::Changed {
                old: 10.0,
                new: 23.0
            },
            diff.size
        );
        assert_eq!(PrimitiveDiff::Unchanged, diff.color);
    }

    #[test]
    fn test_derive_serde() {
        #[derive(Debug, Diff)]
        struct Ball {
            size: f32,
            color: String,
        }
        let a = Ball {
            size: 10.0,
            color: "Red".into(),
        };
        let b = Ball {
            size: 11.0,
            color: "Red".into(),
        };
        let diff = a.diff(&b);
        let expected = r#"{"size":{"old":10.0,"new":11.0,"type":"changed"}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }
}
