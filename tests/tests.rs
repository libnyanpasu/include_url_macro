#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use include_url_macro::{
        include_json_url, include_url, include_url_bytes, include_url_bytes_with_brotli,
    };
    use serde::Deserialize;

    #[test]
    fn test_include_url() {
        let content =
            include_url!("https://raw.githubusercontent.com/rust-lang/rust/master/README.md");
        assert!(content.contains("Rust"));
    }

    #[test]
    fn test_include_bytes() {
        let content =
            include_url_bytes!("https://raw.githubusercontent.com/rust-lang/rust/master/README.md");
        let content = std::str::from_utf8(content).unwrap();
        assert!(content.contains("Rust"));
    }

    #[test]
    fn test_include_bytes_with_brotli() {
        let content = include_url_bytes_with_brotli!(
            "https://raw.githubusercontent.com/rust-lang/rust/master/README.md"
        );
        let mut result = Vec::with_capacity(4096);
        {
            let mut reader = brotli::Decompressor::new(&content[..], 4096);
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf[..]) {
                    Err(e) => {
                        if let std::io::ErrorKind::Interrupted = e.kind() {
                            continue;
                        }
                        panic!("{}", e);
                    }
                    Ok(size) => {
                        if size == 0 {
                            break;
                        }
                        if let Err(e) = result.write_all(&buf[..size]) {
                            panic!("{}", e)
                        }
                    }
                }
            }
        }
        let content = std::str::from_utf8(&result).unwrap();
        assert!(content.contains("Rust"));
    }

    #[test]
    fn test_invalid_url() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/compile-fail/*.rs");
    }

    // Test for generic JSON parsing
    #[test]
    fn test_include_json_url() {
        let json = include_json_url!("https://jsonplaceholder.typicode.com/posts/1");
        assert_eq!(json["userId"].as_i64().unwrap(), 1);
        assert!(json["title"].as_str().unwrap().len() > 0);
        assert!(json["body"].as_str().unwrap().len() > 0);
    }

    // Test for parsing into a specific type
    #[derive(Deserialize, Debug, PartialEq)]
    struct Post {
        userId: i64,
        id: i64,
        title: String,
        body: String,
    }

    #[test]
    fn test_include_json_url_typed() {
        let post = include_json_url!("https://jsonplaceholder.typicode.com/posts/1", Post);
        assert_eq!(post.userId, 1);
        assert_eq!(post.id, 1);
        assert!(!post.title.is_empty());
        assert!(!post.body.is_empty());
    }
}
