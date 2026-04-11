use crate::core::{gen_constructor_with_fallback, implments_impl_mur_dependencies};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	FnArg, GenericArgument, ItemImpl, PathArguments, ReturnType, Type, parse_macro_input,
};

fn extract_result_ok_type(ty: &Type) -> Option<&Type> {
	let Type::Path(type_path) = ty else {
		return None;
	};
	let last = type_path.path.segments.last()?;
	if last.ident != "Result" {
		return None;
	}
	let PathArguments::AngleBracketed(args) = &last.arguments else {
		return None;
	};
	let first = args.args.first()?;
	let GenericArgument::Type(ok_ty) = first else {
		return None;
	};
	Some(ok_ty)
}

fn is_context_type(ty: &Type) -> bool {
	let Type::Path(type_path) = ty else {
		return false;
	};
	type_path
		.path
		.segments
		.last()
		.is_some_and(|seg| seg.ident == "MurRequestContext")
}

pub fn pipe_impl(args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemImpl);
	let impl_type = &input.self_ty;
	let generics = &input.generics;
	let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

	let transform_fn = input.items.iter().find_map(|item| {
		if let syn::ImplItem::Fn(f) = item
			&& f.sig.ident == "transform"
		{
			Some(f)
		} else {
			None
		}
	});

	let Some(transform_fn) = transform_fn else {
		return syn::Error::new_spanned(
			impl_type,
			"Pipe impl must define a `fn transform(...)` method.",
		)
		.to_compile_error()
		.into();
	};

	let output_ty: Type = match &transform_fn.sig.output {
		ReturnType::Type(_, ty) => match extract_result_ok_type(ty) {
			Some(ok_ty) => ok_ty.clone(),
			None => {
				return syn::Error::new_spanned(
					&transform_fn.sig.output,
					"`transform` must return `Result<T, MurError>`.",
				)
				.to_compile_error()
				.into();
			}
		},
		ReturnType::Default => {
			return syn::Error::new_spanned(
				&transform_fn.sig,
				"`transform` must return `Result<T, MurError>`.",
			)
			.to_compile_error()
			.into();
		}
	};

	let input_ty: Type = if args.is_empty() {
		let non_self: Vec<_> = transform_fn
			.sig
			.inputs
			.iter()
			.filter(|a| !matches!(a, FnArg::Receiver(_)))
			.collect();

		match non_self.len() {
			1 => {
				let FnArg::Typed(pat_type) = non_self[0] else {
					return syn::Error::new_spanned(impl_type, "Invalid transform parameter.")
						.to_compile_error()
						.into();
				};
				(*pat_type.ty).clone()
			}
			2 => {
				let FnArg::Typed(pat_type) = non_self[1] else {
					return syn::Error::new_spanned(impl_type, "Invalid transform parameter.")
						.to_compile_error()
						.into();
				};
				(*pat_type.ty).clone()
			}
			_ => {
				return syn::Error::new_spanned(
					&transform_fn.sig,
					"`transform` must have 1 or 2 parameters (excluding &self).",
				)
				.to_compile_error()
				.into();
			}
		}
	} else {
		parse_macro_input!(args as Type)
	};

	let transform_call: TokenStream2 = if is_context_type(&input_ty) {
		quote! { self.transform(ctx) }
	} else {
		quote! { self.transform(ctx, input) }
	};

	let (ensure_constructor, ctor_lets, ctor_args, required_container_typeids) =
		gen_constructor_with_fallback(
			&input,
			Some(
				"Pipe must define `fn new(...) -> Self` in this `#[pipe]` impl block. \
				For pipes with injected services, move the constructor here. \
				For zero-dependency pipes, add: `fn new() -> Self { Self {} }`.",
			),
		);

	let has_new = input
		.items
		.iter()
		.any(|item| matches!(item, syn::ImplItem::Fn(f) if f.sig.ident == "new"));

	let create_expr = quote! { Self::new(#(#ctor_args),*) };
	let _ = has_new;

	let pipe_factory_impl = quote! {
		impl #impl_generics murgamu::MurPipeFactory for #impl_type #where_clause {
			fn __create_factory(
				_injects: &murgamu::MurInjects,
				_container: &murgamu::MurServiceContainer,
			) -> Self {
				#(#ctor_lets)*
				#create_expr
			}
		}
	};

	let pipe_impl_block = quote! {
		impl #impl_generics murgamu::MurPipe<#input_ty> for #impl_type #where_clause {
			type Output = #output_ty;
			type Error = murgamu::MurError;

			fn apply_transform(
				&self,
				ctx: murgamu::MurRequestContext,
				input: #input_ty,
			) -> Result<Self::Output, Self::Error> {
				#transform_call
			}

			fn name(&self) -> &str {
				std::any::type_name::<Self>()
			}

			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}
	};

	let pipe_dyn_impl = quote! {
		impl #impl_generics murgamu::MurPipeDyn for #impl_type #where_clause {
			fn name(&self) -> &str {
				<Self as murgamu::MurPipe<#input_ty>>::name(self)
			}

			fn as_any(&self) -> &dyn std::any::Any {
				<Self as murgamu::MurPipe<#input_ty>>::as_any(self)
			}
		}
	};

	let deps_impl = implments_impl_mur_dependencies(
		required_container_typeids,
		impl_generics,
		impl_type,
		where_clause,
	);

	TokenStream::from(quote! {
		#input

		#ensure_constructor

		#pipe_factory_impl
		#pipe_impl_block
		#pipe_dyn_impl
		#deps_impl
	})
}
