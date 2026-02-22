use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MurRoutePattern {
	pub pattern: String,
	segments: Vec<MurPatternSegment>,
	param_names: Vec<String>,
	segment_count: usize,
	is_static: bool,
	specificity: i32,
}

#[derive(Debug, Clone)]
pub(crate) enum MurPatternSegment {
	Literal(Box<str>),
	Param(Box<str>),
	Wildcard,
	CatchAll(Option<Box<str>>),
}

impl MurRoutePattern {
	pub fn new(pattern: &str) -> Self {
		let pattern = normalize_path(pattern).into_owned();
		let mut segments = Vec::with_capacity(8);
		let mut param_names = Vec::with_capacity(4);
		let mut is_static = true;
		let mut specificity = 0i32;
		for segment in pattern.split('/').filter(|s| !s.is_empty()) {
			if let Some(param_name) = segment.strip_prefix(':') {
				param_names.push(param_name.to_string());
				segments.push(MurPatternSegment::Param(param_name.into()));
				is_static = false;
				specificity += 10;
			} else if segment == "*" {
				segments.push(MurPatternSegment::Wildcard);
				is_static = false;
				specificity += 1;
			} else if segment == "**" {
				segments.push(MurPatternSegment::CatchAll(None));
				is_static = false;
				specificity -= 100;
			} else if let Some(param_name) = segment.strip_prefix('*') {
				param_names.push(param_name.to_string());
				segments.push(MurPatternSegment::CatchAll(Some(param_name.into())));
				is_static = false;
				specificity -= 100;
			} else {
				segments.push(MurPatternSegment::Literal(segment.into()));
				specificity += 100;
			}
		}

		let segment_count = segments.len();

		Self {
			pattern,
			segments,
			param_names,
			segment_count,
			is_static,
			specificity,
		}
	}

	#[inline]
	pub fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
		if self.is_static {
			if normalize_path(path) == self.pattern {
				return Some(HashMap::with_capacity(0));
			}
			return None;
		}

		self.match_path_dynamic(path)
	}

	#[inline(never)]
	fn match_path_dynamic(&self, path: &str) -> Option<HashMap<String, String>> {
		let normalized = normalize_path(path);
		let mut params = HashMap::with_capacity(self.param_names.len());
		let mut path_iter = normalized.split('/').filter(|s| !s.is_empty());
		let mut pattern_idx = 0;

		while pattern_idx < self.segments.len() {
			match &self.segments[pattern_idx] {
				MurPatternSegment::Literal(expected) => match path_iter.next() {
					Some(segment) if segment == expected.as_ref() => {}
					_ => return None,
				},
				MurPatternSegment::Param(name) => match path_iter.next() {
					Some(segment) => {
						params.insert(name.to_string(), segment.to_string());
					}
					None => return None,
				},
				MurPatternSegment::Wildcard => {
					path_iter.next()?;
				}
				MurPatternSegment::CatchAll(name) => {
					let rest: Vec<&str> = path_iter.collect();
					if let Some(param_name) = name {
						params.insert(param_name.to_string(), rest.join("/"));
					}
					return Some(params);
				}
			}
			pattern_idx += 1;
		}

		if path_iter.next().is_none() {
			Some(params)
		} else {
			None
		}
	}

	#[inline]
	pub fn param_names(&self) -> &[String] {
		&self.param_names
	}

	#[inline]
	pub fn has_params(&self) -> bool {
		!self.param_names.is_empty()
	}

	#[inline]
	pub fn segment_count(&self) -> usize {
		self.segment_count
	}

	#[inline]
	pub fn specificity_score(&self) -> i32 {
		self.specificity
	}

	#[inline]
	pub fn is_static(&self) -> bool {
		self.is_static
	}
}

pub(crate) fn normalize_path(path: &str) -> Cow<'_, str> {
	if path.starts_with('/') && (path.len() == 1 || !path.ends_with('/')) {
		return Cow::Borrowed(path);
	}

	let mut normalized = String::with_capacity(path.len() + 1);
	if !path.starts_with('/') {
		normalized.push('/');
	}

	let path_without_trailing = path.trim_end_matches('/');
	normalized.push_str(path_without_trailing);

	if normalized.is_empty() {
		normalized.push('/');
	}

	Cow::Owned(normalized)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pattern_literal() {
		let pattern = MurRoutePattern::new("/users");
		assert!(pattern.match_path("/users").is_some());
		assert!(pattern.match_path("/posts").is_none());
	}

	#[test]
	fn test_pattern_param() {
		let pattern = MurRoutePattern::new("/users/:id");
		let params = pattern.match_path("/users/123").unwrap();
		assert_eq!(params.get("id"), Some(&"123".to_string()));
	}

	#[test]
	fn test_pattern_multiple_params() {
		let pattern = MurRoutePattern::new("/users/:id/posts/:post_id");
		let params = pattern.match_path("/users/123/posts/456").unwrap();
		assert_eq!(params.get("id"), Some(&"123".to_string()));
		assert_eq!(params.get("post_id"), Some(&"456".to_string()));
	}

	#[test]
	fn test_pattern_wildcard() {
		let pattern = MurRoutePattern::new("/files/*");
		assert!(pattern.match_path("/files/doc.txt").is_some());
		assert!(pattern.match_path("/files").is_none());
	}

	#[test]
	fn test_pattern_catch_all() {
		let pattern = MurRoutePattern::new("/files/**");
		assert!(pattern.match_path("/files/a/b/c").is_some());
	}

	#[test]
	fn test_pattern_catch_all_named() {
		let pattern = MurRoutePattern::new("/files/*path");
		let params = pattern.match_path("/files/a/b/c").unwrap();
		assert_eq!(params.get("path"), Some(&"a/b/c".to_string()));
	}

	#[test]
	fn test_normalize_path() {
		assert_eq!(normalize_path("/users"), "/users");
		assert_eq!(normalize_path("/users/"), "/users");
		assert_eq!(normalize_path("users"), "/users");
		assert_eq!(normalize_path(""), "/");
		assert_eq!(normalize_path("/"), "/");
	}

	#[test]
	fn test_specificity() {
		let literal = MurRoutePattern::new("/users/profile");
		let param = MurRoutePattern::new("/users/:id");
		let wildcard = MurRoutePattern::new("/users/*");
		let catch_all = MurRoutePattern::new("/users/**");

		assert!(literal.specificity_score() > param.specificity_score());
		assert!(param.specificity_score() > wildcard.specificity_score());
		assert!(wildcard.specificity_score() > catch_all.specificity_score());
	}

	#[test]
	fn test_static_route_optimization() {
		let pattern = MurRoutePattern::new("/static/route");
		assert!(pattern.is_static());
		assert!(pattern.match_path("/static/route").is_some());
		assert!(pattern.match_path("/static/other").is_none());
	}
}
