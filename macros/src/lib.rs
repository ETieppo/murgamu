mod controller;
mod derive;
mod guards;
mod main_entry;
mod module;
mod response;
mod service;
mod types;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl};

#[proc_macro_attribute]
pub fn controller(args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemImpl);
	controller::controller_impl(args, input).into()
}

#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::get_impl(args, input)
}

#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::post_impl(args, input)
}

#[proc_macro_attribute]
pub fn put(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::put_impl(args, input)
}

#[proc_macro_attribute]
pub fn delete(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::delete_impl(args, input)
}

#[proc_macro_attribute]
pub fn patch(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::patch_impl(args, input)
}

#[proc_macro_attribute]
pub fn head(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::head_impl(args, input)
}

#[proc_macro_attribute]
pub fn options(args: TokenStream, input: TokenStream) -> TokenStream {
	controller::options_impl(args, input)
}

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
	service::service_impl(args, input)
}

#[proc_macro_attribute]
pub fn injectable(args: TokenStream, input: TokenStream) -> TokenStream {
	service::injectable_impl(args, input)
}

#[proc_macro_attribute]
pub fn module(args: TokenStream, input: TokenStream) -> TokenStream {
	module::module_impl(args, input)
}

#[proc_macro_attribute]
pub fn guard(args: TokenStream, input: TokenStream) -> TokenStream {
	guards::guard_impl(args, input)
}

#[proc_macro_attribute]
pub fn interceptor(args: TokenStream, input: TokenStream) -> TokenStream {
	guards::interceptor_impl(args, input)
}

#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::main_impl(args, input)
}

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::route_impl(args, input)
}

#[proc_macro_attribute]
pub fn param(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::param_impl(args, input)
}

#[proc_macro_attribute]
pub fn query(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::query_impl(args, input)
}

#[proc_macro_attribute]
pub fn header(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::header_impl(args, input)
}

#[proc_macro_attribute]
pub fn body(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::body_impl(args, input)
}

#[proc_macro_attribute]
pub fn validate(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::validate_impl(args, input)
}

#[proc_macro_attribute]
pub fn api(args: TokenStream, input: TokenStream) -> TokenStream {
	main_entry::api_impl(args, input)
}

#[proc_macro]
pub fn json_response(input: TokenStream) -> TokenStream {
	response::json_impl(input)
}

#[proc_macro]
pub fn text_response(input: TokenStream) -> TokenStream {
	response::text_impl(input)
}

#[proc_macro]
pub fn html_response(input: TokenStream) -> TokenStream {
	response::html_impl(input)
}

#[proc_macro]
pub fn ok_response(input: TokenStream) -> TokenStream {
	response::ok_impl(input)
}

#[proc_macro]
pub fn no_content_response(input: TokenStream) -> TokenStream {
	response::no_content_impl(input)
}

#[proc_macro_derive(MurDto)]
pub fn derive_dto(input: TokenStream) -> TokenStream {
	derive::derive_dto_impl(input)
}

#[proc_macro_derive(MurEntity)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
	derive::derive_entity_impl(input)
}
