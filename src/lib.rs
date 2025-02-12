use proc_macro::TokenStream;
use quote::quote;
use reqwest::blocking::Client;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, LitStr, Token, Type};
use url::Url;

// Function that contains the core URL fetching logic
pub(crate) fn fetch_url_content(url_str: &str) -> Result<String, String> {
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
        .text()
        .map_err(|e| format!("Failed to read response body: {}", e))
}

// Original include_url macro, now just a thin wrapper
#[proc_macro]
pub fn include_url(input: TokenStream) -> TokenStream {
    let url_str = parse_macro_input!(input as LitStr).value();

    match fetch_url_content(&url_str) {
        Ok(content) => {
            let output = quote! { #content };
            output.into()
        }
        Err(err) => syn::Error::new(proc_macro2::Span::call_site(), err)
            .to_compile_error()
            .into(),
    }
}

// Input parser for json_url macro
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

// JSON-specific macro that uses the fetch_url_content function
#[proc_macro]
pub fn include_json_url(input: TokenStream) -> TokenStream {
    let JsonUrlInput { url, ty } = parse_macro_input!(input as JsonUrlInput);
    let url_str = url.value();

    match fetch_url_content(&url_str) {
        Ok(content) => {
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
