use include_url_macro::include_json_url;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: u32,
}

fn main() {
    // This should fail because the JSONPlaceholder API returns a different structure
    let _user: User = include_json_url!("https://jsonplaceholder.typicode.com/posts/1", User);
}
