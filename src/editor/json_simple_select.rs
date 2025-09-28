use anyhow::Context as _;

pub fn apply(json: &str, selector: &str) -> anyhow::Result<String> {
    let selector = selector
        .split(['.', '[', ']'])
        .filter(|part| !part.is_empty());
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

#[cfg(test)]
#[track_caller]
fn case(input: &str, selector: &str, expected: &str) {
    use std::io::Write as _;

    let actual = apply(input, selector).unwrap();
    assert_eq!(actual, expected);

    let process = std::process::Command::new("jq")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .arg("--compact-output")
        .arg(selector)
        .spawn()
        .expect("jq should be spawnable");
    process
        .stdin
        .as_ref()
        .unwrap()
        .write_all(input.as_bytes())
        .expect("jq process should get input via stdin");
    let output = process.wait_with_output().expect("Should wait for jq");

    if !output.status.success() || !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("jq unsuccessful:\n{stderr}");
    }

    let jq_stdout = String::from_utf8(output.stdout).expect("jq stdout should be Utf-8");
    assert_eq!(jq_stdout.trim(), expected);
}

#[test]
fn simple_object() {
    case(r#"{"foo": {"bar": 42}}"#, ".foo", r#"{"bar":42}"#);
}

#[test]
fn simple_array() {
    case("[13, 37]", ".[1]", "37");
}
