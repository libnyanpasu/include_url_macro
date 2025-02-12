use include_url_macro::include_json_url;

fn main() {
    // This should fail because README.md is not valid JSON
    let _json =
        include_json_url!("https://raw.githubusercontent.com/rust-lang/rust/master/COPYRIGHT");
}
