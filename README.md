# include_url

A Rust procedural macro that fetches URL content at compile time and includes it as a static string.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
include_url_macro = "0.1.0"
```

Then use it in your code:

```rust
use include_url_macro::include_url;

fn main() {
    // Content will be fetched at compile time
    let readme = include_url!("https://raw.githubusercontent.com/rust-lang/rust/master/README.md");
    println!("{}", readme);
}
```

## Features

- Fetches URL content at compile time
- Supports HTTP and HTTPS URLs
- Validates URLs before fetching
- Provides meaningful compile-time errors
- Similar usage to the built-in `include_str!` macro

## Safety

This macro only allows HTTP and HTTPS URLs. It performs URL validation before making any requests.
Note that using this macro will make your build process dependent on network connectivity and the availability of the URLs you're including.

## License

This project is licensed under the MIT License.
