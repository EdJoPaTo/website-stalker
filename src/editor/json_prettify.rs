pub fn prettify(json: &str) -> anyhow::Result<String> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let pretty = serde_json::to_string_pretty(&value)?;
    Ok(pretty)
}

#[test]
fn works() {
    let ugly = r#"{"foo": "bar", "test": 1337 }"#;
    assert_eq!(
        prettify(ugly).unwrap(),
        r#"{
  "foo": "bar",
  "test": 1337
}"#
    );
}
