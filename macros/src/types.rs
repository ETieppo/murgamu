use {proc_macro2::TokenStream, syn::Ident};
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ParamKind {
	SelfRef,
	Context,
	Json(TokenStream),
	Query(TokenStream),
	Path(TokenStream),
	Param(TokenStream),
	Header,
	Body,
	Text,
	Parts,
	RawBody,
	Container,
	// Inject(TokenStream),
	CustomJson(TokenStream),
	Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParamInfo {
	pub name: Ident,
	pub kind: ParamKind,
	pub ty: TokenStream,
	pub is_optional: bool,
}
