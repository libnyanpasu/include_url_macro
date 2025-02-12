#[cfg(test)]
mod tests {
    use include_url_macro::include_url;

    #[test]
    fn test_include_url() {
        let content =
            include_url!("https://raw.githubusercontent.com/rust-lang/rust/master/README.md");
        assert!(content.contains("Rust"));
    }

    #[test]
    fn test_invalid_url() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/compile-fail/*.rs");
    }
}
