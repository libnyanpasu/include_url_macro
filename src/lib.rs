use proc_macro::TokenStream;
use quote::quote;
use reqwest::blocking::Client;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, LitStr, Token, Type};
use url::Url;

#[proc_macro]
pub fn include_url(input: TokenStream) -> TokenStream {
    // Parse the input string literal
    let url_str = parse_macro_input!(input as LitStr).value();

    // Validate URL
    let url = match Url::parse(&url_str) {
        Ok(url) => url,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Invalid URL: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Only allow HTTP(S) schemes
    if url.scheme() != "http" && url.scheme() != "https" {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Only HTTP and HTTPS URLs are supported",
        )
        .to_compile_error()
        .into();
    }

    // Fetch the URL content
    let client = Client::new();
    let response = match client.get(url).send() {
        Ok(response) => response,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to fetch URL: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    let content = match response.text() {
        Ok(text) => text,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to read response body: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate the output tokens
    let output = quote! {
        #content
    };

    output.into()
}
