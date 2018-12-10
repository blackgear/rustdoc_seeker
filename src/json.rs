pub fn fix_json<S: AsRef<str>>(json: S) -> String {
    let mut is_escape = false;
    let mut is_string = false;
    let mut buffer = String::with_capacity(1024);

    for chr in json.as_ref().chars() {
        match chr {
            'N' if !is_string => {
                buffer.push_str("null");
                continue;
            }
            '"' if !is_escape => is_string = !is_string,
            '\\' if !is_escape => is_escape = true,
            _ => is_escape = false,
        };
        buffer.push(chr);
    }

    buffer
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fix_json() {
        assert_eq!(
            r#"[1,null,"is \" N ", "\N"]"#,
            fix_json(r#"[1,N,"is \" N ", "\N"]"#)
        );
    }
}
