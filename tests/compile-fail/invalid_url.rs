use include_url_macro::include_url;

fn main() {
    let _content = include_url!("not_a_url");
}
