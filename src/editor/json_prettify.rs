use serde::Serialize as _;
use serde_json::ser::PrettyFormatter;
use serde_json::Serializer;

pub fn prettify(json: &str) -> anyhow::Result<String> {
    let value: serde_json::Value = serde_json::from_str(json)?;

    let formatter = PrettyFormatter::with_indent(b"\t");
    let mut serializer = Serializer::with_formatter(Vec::new(), formatter);
    value.serialize(&mut serializer)?;
    let pretty =
        String::from_utf8(serializer.into_inner()).expect("serde_json generates only valid Utf-8");
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
