use crate::MurError;
use crate::server::error::builder::MurResult;
use http::request::Parts;
use http_body_util::{BodyExt, LengthLimitError, Limited};
use hyper::body::Bytes;
use hyper::{Request, body::Incoming};

#[derive(Debug)]
pub struct PreprocessedBody {
	pub method: String,
	pub path: String,
	pub parts: Parts,
	pub body_bytes: Option<Bytes>,
}

pub async fn limited_body_extraction(
	req: Request<Incoming>,
	limit: usize,
) -> MurResult<PreprocessedBody> {
	let (parts, body) = req.into_parts();
	let method = parts.method.to_string();
	let path = parts.uri.path().to_string();
	let limited = Limited::new(body, limit);
	let collected = limited.collect().await.map_err(|err| {
		if err.is::<LengthLimitError>() {
			MurError::PayloadTooLarge("--".to_string())
		} else {
			err.into()
		}
	})?;
	let bytes = collected.to_bytes();
	let body_bytes = if bytes.is_empty() { None } else { Some(bytes) };

	Ok(PreprocessedBody {
		method,
		path,
		parts,
		body_bytes,
	})
}
