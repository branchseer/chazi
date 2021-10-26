// A random uuid that marks the stderr output as generated by chazi
pub(crate) const CHAZI_PROBE_SUFFIX: &str = "353887f6-a130-11eb-aad1-54b203047ebd";

pub(crate) fn generate_probe(content: &str) -> String {
    format!("{}_{}{}", content, content.len(), CHAZI_PROBE_SUFFIX)
}

pub(crate) fn parse_probe(line: &str) -> Option<(&str, &str)> {
    let line = line.strip_suffix(CHAZI_PROBE_SUFFIX)?;
    let (line, content_len_str) = line.rsplit_once("_").unwrap();
    let content_len = content_len_str.parse::<usize>().unwrap();
    let line_bytes = line.as_bytes();
    let content_start = line_bytes.len() - content_len;
    let prefix_bytes = &line_bytes[..content_start];
    let content_bytes = &line_bytes[content_start..];
    Some((
        std::str::from_utf8(prefix_bytes).unwrap(),
        std::str::from_utf8(content_bytes).unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{generate_probe, parse_probe};

    #[test]
    fn test_probe() {
        let probe = generate_probe("world");
        let line = format!("hello{}", probe);
        let (prefix, content) = parse_probe(line.as_str()).unwrap();
        assert_eq!(prefix, "hello");
        assert_eq!(content, "world");
    }

    #[test]
    fn test_probe_no_prefix() {
        let probe = generate_probe("42");
        let (prefix, content) = parse_probe(probe.as_str()).unwrap();
        assert_eq!(prefix, "");
        assert_eq!(content, "42");
    }

    #[test]
    fn test_probe_empty_content() {
        let probe = generate_probe("");
        let (prefix, content) = parse_probe(probe.as_str()).unwrap();
        assert_eq!(prefix, "");
        assert_eq!(content, "");

        let with_prefix = format!("b166er{}", probe);
        let (prefix, content) = parse_probe(with_prefix.as_str()).unwrap();
        assert_eq!(prefix, "b166er");
        assert_eq!(content, "");
    }
}
