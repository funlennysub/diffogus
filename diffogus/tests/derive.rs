#[cfg(test)]
mod test {
    use diffogus::diff::{Diffable, PrimitiveDiff};
    use diffogus::Diff;
    use serde::{Deserialize, Serialize};

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
    fn test_derive_serde_simple() {
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
        let expected = r#"{"size":{"type":"changed","value":{"old":10.0,"new":11.0}}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_derive_serde_complex() {
        #[derive(Debug, Diff)]
        struct Box {
            volume: f32,
            color: u8,
            items: Vec<String>,
            open: bool,
        }
        impl Box {
            fn new(volume: f32, color: u8, items: Vec<String>, open: bool) -> Self {
                Self {
                    volume,
                    color,
                    items,
                    open,
                }
            }
        }

        let a = Box::new(10.0, 0, vec!["pen".into(), "mug".into()], false);
        let b = Box::new(10.0, 0, vec!["pen".into()], true);
        let diff = a.diff(&b);
        let expected = r#"{"items":[{"type":"unchanged"},{"type":"removed","value":"mug"}],"open":{"type":"changed","value":{"old":false,"new":true}}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }

    #[test]
    fn test_derive_serde_even_more_complex() {
        #[derive(Default, Debug, PartialEq, Clone, Diff, Serialize, Deserialize)]
        struct Item {
            volume: f32,
            name: String,
        }

        impl Item {
            fn new(volume: f32, name: String) -> Self {
                Self { volume, name }
            }
        }

        #[derive(Default, Debug, PartialEq, Clone, Diff, Serialize, Deserialize)]
        struct Box {
            volume: f32,
            color: u8,
            items: Vec<Item>,
            open: bool,
        }

        impl Box {
            fn new(volume: f32, color: u8, items: Vec<Item>, open: bool) -> Self {
                Self {
                    volume,
                    color,
                    items,
                    open,
                }
            }
        }

        let a = Box::new(10.0, 0, vec![Item::new(5.0, "pen".into())], false);
        let b = Box::new(
            11.0,
            4,
            vec![
                Item::new(5.0, "pen".into()),
                Item::new(12.0, "remote".into()),
            ],
            true,
        );
        let diff = a.diff(&b);
        let expected = r#"{"volume":{"type":"changed","value":{"old":10.0,"new":11.0}},"color":{"type":"changed","value":{"old":0,"new":4}},"items":[{"type":"unchanged"},{"type":"added","value":{"volume":12.0,"name":"remote"}}],"open":{"type":"changed","value":{"old":false,"new":true}}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);

        let a = Box::new(10.0, 0, vec![Item::new(5.0, "pen".into())], false);
        let b = Box::new(11.0, 4, vec![Item::new(12.0, "remote".into())], true);
        let diff = a.diff(&b);
        let expected = r#"{"volume":{"type":"changed","value":{"old":10.0,"new":11.0}},"color":{"type":"changed","value":{"old":0,"new":4}},"items":[{"type":"changed","value":{"volume":{"type":"changed","value":{"old":5.0,"new":12.0}},"name":{"type":"changed","value":{"old":"pen","new":"remote"}}}}],"open":{"type":"changed","value":{"old":false,"new":true}}}"#;
        let diff_str = serde_json::to_string(&diff).unwrap();
        assert_eq!(expected.to_string(), diff_str);
    }
}
