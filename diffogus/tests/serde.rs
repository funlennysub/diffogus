#[cfg(test)]
mod test {
    use diffogus::diff::Diffable;

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
}