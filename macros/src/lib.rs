// Copyright (C) 2026  Emerson Alexandre Tieppo Jr. - Murgamü

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.

// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod controller;
mod core;
mod derive;
mod guard;
mod injectable;
mod interceptor;
mod main_entry;
mod module;
mod pipe;
mod response;
mod service;
mod types;
mod use_pipe;

use proc_macro::TokenStream;
use syn::{ItemImpl, parse_macro_input};

/// Marks an `impl` block as a Murgamu controller and registers its route handlers.
///
/// The optional argument sets the base path prefix for all routes in the block.
/// If omitted, the base path defaults to `"/"`.
///
/// # Example
///
/// ```rust,ignore
/// #[controller("/users")]
/// impl UserController {
///     fn new() -> Self { Self }
///
///     #[get]
///     async fn list(&self) -> MurRes { /* … */ }
///
///     #[post]
///     async fn create(&self, body: CreateUserDto) -> MurRes { /* … */ }
///
///     #[get(":id")]
///     async fn find_one(&self, #[param] id: Param<u64>) -> MurRes { /* … */ }
/// }
/// ```
///
/// Every method annotated with an HTTP verb macro (`#[get]`, `#[post]`, etc.)
/// becomes a route handler. The macro generates the `MurController` and
/// `MurControllerFactory` trait implementations automatically.
#[proc_macro_attribute]
pub fn controller(args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemImpl);
	controller::controller_impl(args, input).into()
}

/// Declares a route handler for `GET` requests.
///
/// The optional argument is the route path relative to the controller's base
/// path. Omitting it (or using `""` / `"/"`) makes the handler respond to
/// the base path itself.
///
/// # Examples
///
/// ```rust,ignore
/// #[get]
/// async fn list(&self) -> MurRes { /* GET /base */ }
///
/// #[get(":id")]
/// async fn find_one(&self, #[param] id: Param<u64>) -> MurRes { /* GET /base/:id */ }
///
/// #[get("/search")]
/// async fn search(&self, #[query] q: SearchDto) -> MurRes { /* GET /base/search */ }
/// ```
#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::get_impl(args, input)
}

/// Declares a route handler for `POST` requests.
///
/// The optional argument is the route path relative to the controller's base path.
///
/// # Example
///
/// ```rust,ignore
/// #[post]
/// async fn create(&self, body: CreateDto) -> MurRes { /* POST /base */ }
///
/// #[post("/bulk")]
/// async fn bulk_create(&self, body: Vec<CreateDto>) -> MurRes { /* POST /base/bulk */ }
/// ```
#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::post_impl(args, input)
}

/// Declares a route handler for `PUT` requests.
///
/// The optional argument is the route path relative to the controller's base path.
///
/// # Example
///
/// ```rust,ignore
/// #[put(":id")]
/// async fn replace(&self, #[param] id: Param<u64>, body: ReplaceDto) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn put(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::put_impl(args, input)
}

/// Declares a route handler for `DELETE` requests.
///
/// The optional argument is the route path relative to the controller's base path.
///
/// # Example
///
/// ```rust,ignore
/// #[delete(":id")]
/// async fn remove(&self, #[param] id: Param<u64>) -> MurRes { /* DELETE /base/:id */ }
/// ```
#[proc_macro_attribute]
pub fn delete(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::delete_impl(args, input)
}

/// Declares a route handler for `PATCH` requests.
///
/// The optional argument is the route path relative to the controller's base path.
///
/// # Example
///
/// ```rust,ignore
/// #[patch(":id")]
/// async fn update(&self, #[param] id: Param<u64>, body: UpdateDto) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn patch(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::patch_impl(args, input)
}

/// Declares a route handler for `HEAD` requests.
///
/// Behaves identically to `#[get]` but the response body is omitted.
/// The optional argument is the route path relative to the controller's base path.
#[proc_macro_attribute]
pub fn head(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::head_impl(args, input)
}

/// Declares a route handler for `OPTIONS` requests.
///
/// The optional argument is the route path relative to the controller's base path.
#[proc_macro_attribute]
pub fn options(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::options_impl(args, input)
}

/// Marks a struct as a Murgamu service and registers it with the DI container.
///
/// Generates the `MurService` and `MurServiceFactory` trait implementations.
/// Fields are resolved automatically from the container at startup.
/// For manual assignment use _ before property, e.g. AppService { _local_generated_id: u32 };
///
/// # Example
///
/// ```rust,ignore
/// #[service]
/// struct UserService {
///     db: DatabaseService,
/// }
/// ```
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
	service::service_impl(args, input)
}

/// Marks a field as an injected dependency.
///
/// Used inside structs annotated with `#[service]`, `#[guard]`,
/// `#[interceptor]`, or `#[controller]` to declare a dependency that the
/// framework resolves from the DI container.
///
/// ```rust,ignore
/// #[service]
/// struct ReportService {
///     db: DatabaseService,
///     cache: CacheService,
/// }
/// ```
#[proc_macro_attribute]
pub fn inject(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Marks a struct as an injectable value that can be registered with `.inject()`.
///
/// `#[injectable]` is the lightweight counterpart to `#[service]`. Use it for
/// configuration structs, external clients, or shared state that does not need
/// the full service lifecycle (`on_init` / `on_shutdown`).
///
/// # Example
///
/// ```rust,ignore
/// #[injectable]
/// struct AppConfig {
///     pub database_url: String,
///     pub jwt_secret: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn injectable(args: TokenStream, input: TokenStream) -> TokenStream {
	injectable::injectable_impl(args, input)
}

/// Declares a Murgamu module that groups controllers, services, and imports.
///
/// Accepts a comma-separated list of sections:
///
/// - `controllers: [T, …]` — controllers registered in this module.
/// - `providers: [T, …]` — services available within this module.
/// - `imports: [M::new(), …]` — other modules whose exports are visible here.
/// - `exports: [T, …]` — services made available to importing modules.
///
/// # Example
///
/// ```rust,ignore
/// #[module(
///     controllers: [UserController, AuthController],
///     providers:   [UserService, AuthService],
///     imports:     [DatabaseModule::new()],
///     exports:     [UserService],
/// )]
/// struct AppModule;
/// ```
#[proc_macro_attribute]
pub fn module(args: TokenStream, input: TokenStream) -> TokenStream {
	module::module_impl(args, input)
}

/// Marks a struct as a Murgamu route guard.
///
/// Generates the `MurGuard` and `MurGuardFactory` trait implementations.
/// Guards protect routes from unauthorized access; they are evaluated before
/// the route handler runs.
/// For manual assignment use _ before property, e.g. AppGuard { _local_generated_id: u32 };
///
/// # Example
///
/// ```rust,ignore
/// #[guard]
/// struct JwtGuard {
///     auth: AuthService,
/// }
///
/// impl MurGuard for JwtGuard {
///     fn check_can_activate<'a>(&'a self, ctx: &'a MurRequestContext) -> MurGuardFuture<'a> {
///         Box::pin(async move {
///             self.auth.validate_token(ctx.header("Authorization")).await
///         })
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn guard(args: TokenStream, input: TokenStream) -> TokenStream {
	guard::guard_impl(args, input)
}

/// Marks a struct as a Murgamu transformation pipe.
///
/// Pipes transform handler parameters before they reach the handler body.
/// Generates the `MurPipe` and `MurPipeFactory` trait implementations.
///
/// # Example
///
/// ```rust,ignore
/// #[pipe]
/// struct TrimPipe;
///
/// impl MurPipe<MurText> for TrimPipe {
///     type Output = String;
///
///     fn transform(&self, input: MurText, _ctx: MurRequestContext) -> Result<String, MurError> {
///         Ok(input.into_inner().trim().to_string())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn pipe(args: TokenStream, input: TokenStream) -> TokenStream {
	pipe::pipe_impl(args, input)
}

/// Marks a struct as a Murgamu interceptor.
///
/// Interceptors wrap individual route handler invocations, running logic
/// before and/or after the handler. Generates the `MurInterceptor` and
/// `MurInterceptorFactory` trait implementations.
///
/// # Example
///
/// ```rust,ignore
/// #[interceptor]
/// struct TimingInterceptor;
///
/// impl MurInterceptor for TimingInterceptor {
///     fn before(&self, _ctx: &MurRequestContext) -> MurInterceptorFuture {
///         Box::pin(async { Ok(()) })
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn interceptor(args: TokenStream, input: TokenStream) -> TokenStream {
	interceptor::interceptor_impl(args, input)
}

/// Sets up the Tokio async runtime for the application entry point.
///
/// Place `#[murgamu::main]` on an `async fn main()` to replace the standard
/// `#[tokio::main]` annotation with Murgamu's runtime wrapper.
///
/// # Example
///
/// ```rust,ignore
/// #[murgamu::main]
/// async fn main() -> MurMainResult {
///     MurServer::new()
///         .module(AppModule::new())
///         .bind(("0.0.0.0", 3000))?
///         .run()
///         .await
/// }
/// ```
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::main_impl(args, input)
}

/// Declares a route handler for a custom or non-standard HTTP method.
///
/// Use `#[route]` when none of the standard verb macros (`#[get]`, `#[post]`,
/// etc.) fits the desired method.
///
/// # Example
///
/// ```rust,ignore
/// #[route("PURGE", "/cache")]
/// async fn purge_cache(&self) -> MurRes {
///     mur_json!(serde_json::json!({ "purged": true }))
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::route_impl(args, input)
}

/// Marks a handler parameter as a single URL path segment.
///
/// When applied to a plain type (e.g. `String`, `u64`), the parameter is
/// extracted from the matching named segment in the route pattern.
///
/// Prefer [`MurPath<T>`](crate::MurPath) for structs that capture multiple
/// segments at once.
///
/// # Example
///
/// ```rust,ignore
/// #[get("/users/:id")]
/// async fn get_user(&self, #[param] id: u64) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn param(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::param_impl(args, input)
}

/// Marks a handler parameter as a deserialized query string.
///
/// The query string is parsed with `serde_urlencoded` and the result is
/// deserialized directly into the annotated type (no `MurQuery<T>` wrapper).
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct Filters { page: Option<u32>, q: Option<String> }
///
/// #[get("/items")]
/// async fn list(&self, #[query] filters: Filters) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn query(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::query_impl(args, input)
}

/// Marks a handler parameter as a single named query string value.
///
/// The named key is extracted from the query string and parsed into the
/// annotated type. Returns an error if the key is absent or unparseable.
///
/// # Example
///
/// ```rust,ignore
/// #[get("/search")]
/// async fn search(&self, #[queryparam] q: String) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn queryparam(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::query_impl(args, input)
}

/// Marks a handler parameter as a request header value.
///
/// The header name is derived from the parameter name. The value is provided
/// as a raw `String`; parsing is the caller's responsibility.
///
/// # Example
///
/// ```rust,ignore
/// #[get("/protected")]
/// async fn protected(&self, #[header] authorization: String) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn header(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::header_impl(args, input)
}

/// Marks a handler parameter as the deserialized JSON request body.
///
/// The body bytes are deserialized directly into the annotated type using
/// `serde_json` (no `MurJson<T>` wrapper). The type must implement
/// `serde::Deserialize`.
///
/// # Example
///
/// ```rust,ignore
/// #[post("/users")]
/// async fn create(&self, #[body] dto: CreateUserDto) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn body(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::body_impl(args, input)
}

/// Triggers automatic validation of a handler parameter before execution.
///
/// The annotated type must implement a `validate(&self) -> Result<(), String>`
/// method (generated automatically by `#[derive(MurDto)]`). If validation
/// fails, a `400 Bad Request` response is returned before the handler runs.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(MurDto, Deserialize)]
/// struct CreateUserDto {
///     pub name: String,
///     pub email: String,
/// }
///
/// #[post("/users")]
/// async fn create(&self, #[validate] dto: CreateUserDto) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn validate(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::validate_impl(args, input)
}

/// Attaches OpenAPI metadata to a route handler.
///
/// The metadata is used when generating API documentation. All fields are
/// optional; omit any that are not relevant to the endpoint.
///
/// # Example
///
/// ```rust,ignore
/// #[api(
///     summary     = "List all users",
///     description = "Returns a paginated list of registered users.",
///     tags        = ["users"],
///     responses   = [(200, "Paginated user list"), (401, "Unauthorized")],
/// )]
/// #[get("/users")]
/// async fn list(&self) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn api(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::api_impl(args, input)
}

/// Constructs a `200 OK` JSON response from an expression.
///
/// The expression is serialized with `serde_json` and wrapped in an `Ok(…)`
/// result. Equivalent to `MurHttpResponse::ok().json(expr)`.
///
/// # Example
///
/// ```rust,ignore
/// #[get("/health")]
/// async fn health(&self) -> MurRes {
///     json_response!({ "status": "ok" })
/// }
/// ```
#[proc_macro]
pub fn json_response(input: TokenStream) -> TokenStream {
	response::json_impl(input)
}

/// Constructs a `200 OK` plain-text response from an expression.
///
/// The expression must evaluate to a type that implements `Into<String>`.
/// Equivalent to `MurHttpResponse::ok().text(expr)`.
///
/// # Example
///
/// ```rust,ignore
/// text_response!("Hello, world!")
/// ```
#[proc_macro]
pub fn text_response(input: TokenStream) -> TokenStream {
	response::text_impl(input)
}

/// Constructs a `200 OK` HTML response from an expression.
///
/// The expression must evaluate to a type that implements `Into<String>`.
/// Sets `Content-Type: text/html; charset=utf-8` automatically.
///
/// # Example
///
/// ```rust,ignore
/// html_response!("<h1>Hello</h1>")
/// ```
#[proc_macro]
pub fn html_response(input: TokenStream) -> TokenStream {
	response::html_impl(input)
}

/// Constructs a `200 OK` response with an empty body.
///
/// Equivalent to `MurHttpResponse::ok().empty()`.
#[proc_macro]
pub fn ok_response(input: TokenStream) -> TokenStream {
	response::ok_impl(input)
}

/// Constructs a `204 No Content` response with an empty body.
///
/// Equivalent to `MurHttpResponse::no_content()`.
#[proc_macro]
pub fn no_content_response(input: TokenStream) -> TokenStream {
	response::no_content_impl(input)
}

/// Derives a default `validate` method on a DTO struct.
///
/// The generated implementation returns `Ok(())` unconditionally. Override it
/// manually or use custom validation logic when field-level constraints are
/// needed.
///
/// The derive also enables the struct to be used with the `#[validate]`
/// parameter attribute on route handlers.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(MurDto, Deserialize)]
/// pub struct CreateUserDto {
///     pub name: String,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(MurDto)]
pub fn derive_dto(input: TokenStream) -> TokenStream {
	derive::derive_dto_impl(input)
}

/// Derives a thin entity helper on a struct.
///
/// When the struct has a field named `id: String`, the derive generates an
/// `id(&self) -> &str` accessor. For structs without an `id` field the derive
/// is a no-op.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(MurEntity)]
/// pub struct User {
///     pub id: String,
///     pub name: String,
/// }
///
/// let user = User { id: "abc".into(), name: "Alice".into() };
/// assert_eq!(user.id(), "abc");
/// ```
#[proc_macro_derive(MurEntity)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
	derive::derive_entity_impl(input)
}

/// Marks a route as publicly accessible, bypassing global authentication guards.
///
/// Without this attribute every route requires the caller to pass all
/// registered global guards. Applying `#[public]` exempts the route from
/// those checks while still allowing per-route guards.
///
/// # Example
///
/// ```rust,ignore
/// #[public]
/// #[post("/auth/login")]
/// async fn login(&self, #[body] dto: LoginDto) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn public(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Restricts a route to callers that hold one or more specific roles.
///
/// Accepts a comma-separated list of role identifiers. The route's access
/// control list is checked after all guards pass.
///
/// # Example
///
/// ```rust,ignore
/// #[role(UserEnum::Admin, UserEnum::Moderator)]
/// #[delete("/users/:id")]
/// async fn delete_user(&self, #[param] id: Param<u64>) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn role(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}

/// Applies a transformation pipe to a handler parameter.
///
/// The argument must be the concrete pipe type. The pipe's `apply_transform`
/// is called with the [`MurRequestContext`](murgamu::MurRequestContext) before
/// the handler runs, and the transformed value is injected as the parameter.
///
/// # Example
///
/// ```rust,ignore
/// #[post("/upload")]
/// async fn upload(&self, #[use_pipe(FileSizePipe)] file: FileData) -> MurRes { /* … */ }
/// ```
#[proc_macro_attribute]
pub fn use_pipe(args: TokenStream, input: TokenStream) -> TokenStream {
	use_pipe::use_pipe_impl(args, input)
}
