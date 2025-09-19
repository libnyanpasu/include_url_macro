#![feature(proc_macro_tracked_env)]

//! A procedural macro crate for including URL content as static strings at compile time.
//!
//! This crate provides two main macros:
//! - [`include_url!`] for including raw content from URLs
//! - [`include_json_url!`] for including and parsing JSON content from URLs
//!
//! # Examples
//!
//! Basic usage with text content:
//! ```rust
//! use include_url_macro::include_url;
//!
//! const CONTENT: &str = include_url!("https://example.com/static/content.txt");
//! ```
//!
//! Including JSON content with type inference:
//! ```rust
//! use include_url_macro::include_json_url;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Post {
//!   userId: i32,
//!   id: i32,
//!   title: String,
//!   body: String,
//! }
//!
//! let post: Post = include_json_url!("https://jsonplaceholder.typicode.com/posts/1", Post);
//! ```

use std::{fs::OpenOptions, io::Write};

use proc_macro::TokenStream;
use quote::quote;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::env;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, LitStr, Token, Type};
use url::Url;

/// Fetches content from a URL at compile time.
///
/// # Arguments
///
/// * `url_str` - The URL to fetch content from
///
/// # Returns
///
/// * `Ok(String)` - The content fetched from the URL
/// * `Err(String)` - A descriptive error message if the fetch failed
///
/// # Security
///
/// This function only supports HTTP and HTTPS URLs to prevent potential security issues
/// with other URL schemes.
pub(crate) fn fetch_url_content(url_str: &str) -> Result<bytes::Bytes, String> {
    // Validate URL
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // Only allow HTTP(S) schemes
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS URLs are supported".to_string());
    }

    // Fetch the URL content
    let client = Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "include_url_macro")
        .send()
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    response
        .bytes()
        .map_err(|e| format!("Failed to read response body: {}", e))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum CompressKind {
    #[default]
    None,
    #[cfg(feature = "brotli")]
    Brotli,
}

pub(crate) fn cached_url_content(
    url_str: &str,
    compress_kind: CompressKind,
) -> Result<std::path::PathBuf, String> {
    let out_dir = std::path::Path::new(env!("INCLUDE_URL_CACHE_DIR"));
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }
    let crate_name =
        proc_macro::tracked_env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown".into());
    let mut hasher = Sha256::new();
    hasher.update(crate_name.as_bytes());
    hasher.update(b"\0");
    hasher.update(url_str.as_bytes());
    hasher.update(b"\0");
    hasher.update(format!("{:?}", compress_kind));
    let hash = hasher.finalize();
    let filename = format!("{:x}", hash);
    let cache_file = out_dir.join(filename);
    if cache_file.exists() {
        return Ok(cache_file);
    }

    let content = fetch_url_content(url_str)?;

    let content = match compress_kind {
        CompressKind::None => content,
        #[cfg(feature = "brotli")]
        CompressKind::Brotli => {
            let mut buffer = Vec::with_capacity(4096);
            {
                let mut encoder = brotli::CompressorWriter::new(&mut buffer, 4096, 11, 22);
                encoder
                    .write_all(&content)
                    .map_err(|e| format!("Failed to write compressed content: {}", e))?;
                encoder
                    .flush()
                    .map_err(|e| format!("Failed to flush compressed content: {}", e))?;
            }
            bytes::Bytes::from(buffer)
        }
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&cache_file)
        .map_err(|e| format!("Failed to open cache file: {}", e))?;

    file.write_all(&content)
        .map_err(|e| format!("Failed to write cache file: {}", e))?;
    Ok(cache_file)
}

/// A procedural macro that includes content from a URL as a static string at compile time.
///
/// # Usage
///
/// ```rust
/// use include_url_macro::include_url;
///
/// const STATIC_CONTENT: &str = include_url!("https://example.com/static/content.txt");
/// ```
///
/// # Errors
///
/// This macro will fail at compile time if:
/// * The URL is invalid
/// * The URL scheme is not HTTP or HTTPS
/// * The content cannot be fetched
/// * The response is not valid UTF-8
#[proc_macro]
pub fn include_url(input: TokenStream) -> TokenStream {
    let url_str = parse_macro_input!(input as LitStr).value();

    match cached_url_content(&url_str, CompressKind::None) {
        Ok(path) => {
            let path_str = path.display().to_string();
            let output = quote! { include_str!(#path_str) };
            output.into()
        }
        Err(err) => syn::Error::new(proc_macro2::Span::call_site(), err)
            .to_compile_error()
            .into(),
    }
}

#[proc_macro]
pub fn include_url_bytes(input: TokenStream) -> TokenStream {
    let url_str = parse_macro_input!(input as LitStr).value();

    match cached_url_content(&url_str, CompressKind::None) {
        Ok(path) => {
            let path_str = path.display().to_string();
            let output = quote! { include_bytes!(#path_str) };
            output.into()
        }
        Err(err) => syn::Error::new(proc_macro2::Span::call_site(), err)
            .to_compile_error()
            .into(),
    }
}

#[cfg(feature = "brotli")]
#[proc_macro]
pub fn include_url_bytes_with_brotli(input: TokenStream) -> TokenStream {
    let url_str = parse_macro_input!(input as LitStr).value();

    match cached_url_content(&url_str, CompressKind::Brotli) {
        Ok(path) => {
            let path_str = path.display().to_string();
            let output = quote! { include_bytes!(#path_str) };
            output.into()
        }
        Err(err) => syn::Error::new(proc_macro2::Span::call_site(), err)
            .to_compile_error()
            .into(),
    }
}

/// Parser for the `include_json_url` macro's input.
///
/// Handles both the URL and optional type specification.
struct JsonUrlInput {
    url: LitStr,
    ty: Option<Type>,
}

impl Parse for JsonUrlInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let url = input.parse()?;

        // Check if there's a type specification after a comma
        let ty = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(JsonUrlInput { url, ty })
    }
}

/// A procedural macro that includes and parses JSON content from a URL at compile time.
///
/// This macro can either return a generic `serde_json::Value` or parse the JSON into
/// a specific type that implements `serde::Deserialize`.
///
/// # Usage
///
/// Basic usage (returns `serde_json::Value`):
/// ```rust
/// use include_url_macro::include_json_url;
///
/// let json = include_json_url!("https://jsonplaceholder.typicode.com/posts");
/// ```
///
/// Usage with a specific type:
/// ```rust
/// use include_url_macro::include_json_url;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Post {
///   userId: i32,
///   id: i32,
///   title: String,
///   body: String,
/// }
///
/// let post: Post = include_json_url!("https://jsonplaceholder.typicode.com/posts/1", Post);
/// ```
///
/// # Errors
///
/// This macro will fail at compile time if:
/// * The URL is invalid
/// * The URL scheme is not HTTP or HTTPS
/// * The content cannot be fetched
/// * The response is not valid JSON
/// * The JSON cannot be parsed into the specified type (if a type is provided)
#[proc_macro]
pub fn include_json_url(input: TokenStream) -> TokenStream {
    let JsonUrlInput { url, ty } = parse_macro_input!(input as JsonUrlInput);
    let url_str = url.value();

    match cached_url_content(&url_str, CompressKind::None) {
        Ok(path) => {
            let content = match std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to open cache file: {}", e))
            {
                Ok(content) => content,
                Err(e) => {
                    return syn::Error::new(proc_macro2::Span::call_site(), e)
                        .to_compile_error()
                        .into()
                }
            };
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => {
                    // JSON is valid, proceed with the original logic
                    let output = match ty {
                        Some(ty) => quote! {{
                            let json_str = #content;
                            serde_json::from_str::<#ty>(&json_str)
                                .expect("Failed to parse JSON into the specified type")
                        }},
                        None => quote! {{
                            let json_str = #content;
                            serde_json::from_str::<serde_json::Value>(&json_str)
                                .expect("Failed to parse JSON")
                        }},
                    };
                    output.into()
                }
                Err(json_err) => {
                    // Return a compile error if JSON is invalid
                    syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Invalid JSON content from URL: {}", json_err),
                    )
                    .to_compile_error()
                    .into()
                }
            }
        }
        Err(err) => syn::Error::new(proc_macro2::Span::call_site(), err)
            .to_compile_error()
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that valid URLs can be fetched
    #[test]
    fn test_fetch_url_content() {
        let result = fetch_url_content("https://example.com");
        assert!(result.is_ok());
    }

    /// Test that invalid URL schemes are rejected
    #[test]
    fn test_invalid_scheme() {
        let result = fetch_url_content("ftp://example.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Only HTTP and HTTPS URLs are supported"));
    }

    /// Test that invalid URLs are rejected
    #[test]
    fn test_invalid_url() {
        let result = fetch_url_content("not-a-url");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL"));
    }
}
