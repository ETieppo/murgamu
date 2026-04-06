use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

/// Macro to create a JSON response with simplified syntax.
///
/// This macro allows creating JSON responses without manually calling
/// `MurHttpResponse::json()` and wrapping in `json!()`.
///
/// # Example
/// ```ignore
/// // Instead of:
/// MurHttpResponse::json(json!({ "message": "Hello" }))
///
/// // You can write:
/// json_response!({ "message": "Hello" })
/// ```
pub fn json_impl(input: TokenStream) -> TokenStream {
	// Convert TokenStream to TokenStream2 and pass through
	let input: TokenStream2 = input.into();
	let output = quote::quote! {
		MurHttpResponse::json(serde_json::json!(#input))
	};

	TokenStream::from(output)
}

/// Macro to create a text response with simplified syntax.
///
/// # Example
/// ```ignore
/// text_response!("Hello, World!")
/// ```
pub fn text_impl(input: TokenStream) -> TokenStream {
	let input: TokenStream2 = input.into();
	let output = quote::quote! {
		MurHttpResponse::text(#input)
	};

	TokenStream::from(output)
}

/// Macro to create an HTML response with simplified syntax.
///
/// # Example
/// ```ignore
/// html_response!("<h1>Hello, World!</h1>")
/// ```
pub fn html_impl(input: TokenStream) -> TokenStream {
	let input: TokenStream2 = input.into();
	let output = quote::quote! {
		MurHttpResponse::html(#input)
	};

	TokenStream::from(output)
}

/// Macro to create a 200 OK response.
///
/// # Example
/// ```ignore
/// ok_response!()
/// ```
pub fn ok_impl(_input: TokenStream) -> TokenStream {
	let output = quote::quote! {
		MurHttpResponse::ok().build()
	};

	TokenStream::from(output)
}

/// Macro to create a 204 No Content response.
///
/// # Example
/// ```ignore
/// no_content_response!()
/// ```
pub fn no_content_impl(_input: TokenStream) -> TokenStream {
	let output = quote::quote! {
		MurHttpResponse::no_content()
	};

	TokenStream::from(output)
}
