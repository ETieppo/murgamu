use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Entry point macro for Murgamu applications.
///
/// The `#[murgamu::main]` macro wraps your main function with `#[tokio::main]`,
/// setting up the async runtime for your server.
///
/// # Example
/// ```ignore
/// #[murgamu::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     MurServer::new()
///         .module(AppModule::new())
///         .bind("127.0.0.1:3000")?
///         .run()
///         .await
/// }
/// ```
///
/// # Generated Code
/// The macro expands to:
/// ```ignore
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // your code here
/// }
/// ```
///
/// # Features
/// - Automatically sets up the Tokio async runtime
/// - Supports the standard async main function pattern
/// - Compatible with all Tokio features (multi-thread, current-thread)
pub fn main_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemFn);
	let output = generate_main_impl(&input);
	TokenStream::from(output)
}

fn generate_main_impl(input: &ItemFn) -> TokenStream2 {
	let attrs = &input.attrs;
	let vis = &input.vis;
	let sig = &input.sig;
	let block = &input.block;
	if sig.asyncness.is_none() {
		let error_msg = "ERROR: #[murgamu::main] requires an async function";
		return syn::Error::new_spanned(sig, error_msg).to_compile_error();
	}

	quote! {
		#(#attrs)*
		#[tokio::main]
		#vis #sig {
			#block
		}
	}
}

/// Generic route macro for custom HTTP methods.
///
/// The `#[route]` macro allows you to specify any HTTP method for a route.
/// For standard methods, prefer using `#[get]`, `#[post]`, etc.
///
/// # Example
/// ```ignore
/// #[route("CUSTOM", "/custom-endpoint")]
/// async fn custom_handler(&self) -> MurRes {
///     MurRes::json(json!({ "method": "CUSTOM" }))
/// }
/// ```
///
/// # Arguments
/// - First argument: HTTP method as a string (e.g., "GET", "POST", "CUSTOM")
/// - Second argument: Route path (e.g., "/users", "/users/:id")
pub fn route_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	// Pass through - processed by controller macro
	input
}

/// Macro to mark a parameter as extracted from path parameters.
///
/// This is a marker macro that indicates a parameter should be
/// extracted from the URL path.
///
/// # Example
/// ```ignore
/// #[get("/users/:id")]
/// async fn get_user(&self, #[param] id: String) -> MurRes {
///     // id is extracted from the :id path parameter
/// }
/// ```
///
/// # Note
/// In most cases, you don't need this macro. Use `MurPath<T>` instead:
/// ```ignore
/// #[get("/users/:id")]
/// async fn get_user(&self, params: MurPath<UserParams>) -> MurRes {
///     let id = &params.id;
/// }
/// ```
pub fn param_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Macro to mark a parameter as extracted from query parameters.
///
/// This is a marker macro that indicates a parameter should be
/// extracted from the URL query string.
///
/// # Example
/// ```ignore
/// #[get("/users")]
/// async fn list_users(&self, #[query] page: Option<i32>) -> MurRes {
///     // page is extracted from ?page=X
/// }
/// ```
///
/// # Note
/// In most cases, use `MurQuery<T>` instead:
/// ```ignore
/// #[get("/users")]
/// async fn list_users(&self, query: MurQuery<ListQuery>) -> MurRes {
///     let page = query.page.unwrap_or(1);
/// }
/// ```
pub fn query_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Macro to mark a parameter as extracted from headers.
///
/// This is a marker macro that indicates a parameter should be
/// extracted from request headers.
///
/// # Example
/// ```ignore
/// #[get("/protected")]
/// async fn protected(&self, #[header("Authorization")] auth: String) -> MurRes {
///     // auth contains the Authorization header value
/// }
/// ```
///
/// # Note
/// You can also use `MurRequestContext` for header access:
/// ```ignore
/// #[get("/protected")]
/// async fn protected(&self, ctx: MurRequestContext) -> MurRes {
///     let auth = ctx.header("Authorization");
/// }
/// ```
pub fn header_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Macro to mark a parameter as extracted from the request body.
///
/// This is a marker macro that indicates a parameter should be
/// extracted from the request body.
///
/// # Example
/// ```ignore
/// #[post("/users")]
/// async fn create_user(&self, #[body] data: CreateUserDto) -> MurRes {
///     // data is parsed from the JSON body
/// }
/// ```
///
/// # Note
/// In most cases, just use the type directly:
/// ```ignore
/// #[post("/users")]
/// async fn create_user(&self, data: CreateUserDto) -> MurRes {
///     // data is automatically parsed from JSON
/// }
/// ```
pub fn body_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Macro to mark a parameter for automatic validation.
///
/// The `#[validate]` macro enables automatic validation of request data.
/// The type must implement a `validate()` method.
///
/// # Example
/// ```ignore
/// #[derive(MurDto, Deserialize)]
/// pub struct CreateUserDto {
///     pub name: String,
///     pub email: String,
/// }
///
/// impl CreateUserDto {
///     pub fn validate(&self) -> Result<(), String> {
///         if self.name.is_empty() {
///             return Err("Name cannot be empty".to_string());
///         }
///         if !self.email.contains('@') {
///             return Err("Invalid email".to_string());
///         }
///         Ok(())
///     }
/// }
///
/// #[post("/users")]
/// async fn create_user(&self, #[validate] data: CreateUserDto) -> MurRes {
///     // data is validated before this code runs
/// }
/// ```
///
/// # Validation Errors
/// If validation fails, a 400 Bad Request response is automatically
/// returned with the validation error message.
pub fn validate_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Macro to add API documentation to a route.
///
/// The `#[api]` macro adds metadata to a route for OpenAPI documentation
/// generation.
///
/// # Example
/// ```ignore
/// #[api(
///     summary = "List all users",
///     description = "Returns a paginated list of users",
///     tags = ["users"],
///     responses = [
///         (200, "List of users"),
///         (401, "Unauthorized"),
///     ]
/// )]
/// #[get("/users")]
/// async fn list_users(&self) -> MurRes {
///     // ...
/// }
/// ```
///
/// # Attributes
/// - `summary` - Short description of the endpoint
/// - `description` - Detailed description
/// - `tags` - Tags for grouping in documentation
/// - `responses` - List of possible responses with status codes
/// - `deprecated` - Mark the endpoint as deprecated
pub fn api_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

// Macro to mark an endpoint as deprecated.
//
// The `#[deprecated]` macro marks an endpoint as deprecated in the
// API documentation and optionally adds a deprecation notice.
//
// # Example
// ```ignore
// #[deprecated("Use /api/v2/users instead")]
// #[get("/api/v1/users")]
// async fn list_users_v1(&self) -> MurRes {
//     // Old implementation
// }
// ```
// ///
// /// # OpenAPI Integration
// /// When OpenAPI documentation is generated, deprecated endpoints
// /// are marked with the `deprecated: true` flag.
// pub fn deprecated_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
// 	input
// }
