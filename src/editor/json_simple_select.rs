use anyhow::Context as _;

pub fn apply(json: &str, selector: &str) -> anyhow::Result<String> {
    let selector = selector.split('.').filter(|part| !part.is_empty());
    let value: serde_json::Value = serde_json::from_str(json)?;
    let mut current = &value;
    for part in selector {
        match current {
            serde_json::Value::Null => anyhow::bail!("can not select into a null value"),
            serde_json::Value::Bool(_) => anyhow::bail!("can not select into a bool value"),
            serde_json::Value::Number(_) => anyhow::bail!("can not select into a numeric value"),
            serde_json::Value::String(_) => anyhow::bail!("can not select into a string value"),
            serde_json::Value::Array(values) => {
                let index = part
                    .parse::<usize>()
                    .context("selector can not be used to index an array")?;
                current = values
                    .get(index)
                    .context("selector selected out of bounds in the array")?;
            }
            serde_json::Value::Object(map) => {
                current = map
                    .get(part)
                    .context("selector tried to select a non existing key in object")?;
            }
        }
    }
    let output = serde_json::to_string(current)?;
    Ok(output)
}

#[test]
fn simple_object() {
    let input = r#"{"foo": {"bar": 42}}"#;
    let selector = ".foo";
    let expected = r#"{"bar":42}"#;
    let actual = apply(input, selector).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn simple_array() {
    let input = "[13, 37]";
    let selector = ".1";
    let expected = "37";
    let actual = apply(input, selector).unwrap();
    assert_eq!(actual, expected);
}
