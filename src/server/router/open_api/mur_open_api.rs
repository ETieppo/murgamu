use super::{
	components::MurApiComponents, contact::MurApiContact, external_doc::MurApiExternalDocs,
	info::MurApiInfo, license::MurApiLicense, path_item::MurApiPathItem, schema::MurApiSchema,
	security_scheme::MurApiSecurityScheme, server::MurApiServer, spec::MurOpenApiSpec,
	tag::MurApiTag,
};
use std::collections::HashMap;

pub struct MurOpenApi {
	spec: MurOpenApiSpec,
}

impl MurOpenApi {
	pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
		Self {
			spec: MurOpenApiSpec {
				openapi: "3.0.3".to_string(),
				info: MurApiInfo {
					title: title.into(),
					version: version.into(),
					..Default::default()
				},
				..Default::default()
			},
		}
	}

	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.spec.info.description = Some(desc.into());
		self
	}

	pub fn terms_of_service(mut self, url: impl Into<String>) -> Self {
		self.spec.info.terms_of_service = Some(url.into());
		self
	}

	pub fn contact(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
		self.spec.info.contact = Some(MurApiContact {
			name: Some(name.into()),
			email: Some(email.into()),
			url: None,
		});
		self
	}

	pub fn contact_full(
		mut self,
		name: impl Into<String>,
		email: impl Into<String>,
		url: impl Into<String>,
	) -> Self {
		self.spec.info.contact = Some(MurApiContact {
			name: Some(name.into()),
			email: Some(email.into()),
			url: Some(url.into()),
		});
		self
	}

	pub fn license(mut self, name: impl Into<String>) -> Self {
		self.spec.info.license = Some(MurApiLicense {
			name: name.into(),
			url: None,
		});
		self
	}

	pub fn license_url(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
		self.spec.info.license = Some(MurApiLicense {
			name: name.into(),
			url: Some(url.into()),
		});
		self
	}

	pub fn server(mut self, url: impl Into<String>, description: impl Into<String>) -> Self {
		self.spec.servers.push(MurApiServer {
			url: url.into(),
			description: Some(description.into()),
			variables: HashMap::new(),
		});
		self
	}

	pub fn tag(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
		self.spec.tags.push(MurApiTag {
			name: name.into(),
			description: Some(description.into()),
			external_docs: None,
		});
		self
	}

	pub fn path<F>(mut self, path: impl Into<String>, builder: F) -> Self
	where
		F: FnOnce(MurApiPathItem) -> MurApiPathItem,
	{
		let path_item = builder(MurApiPathItem::default());
		self.spec.paths.insert(path.into(), path_item);
		self
	}

	pub fn path_item(mut self, path: impl Into<String>, item: MurApiPathItem) -> Self {
		self.spec.paths.insert(path.into(), item);
		self
	}

	pub fn schema(mut self, name: impl Into<String>, schema: MurApiSchema) -> Self {
		if self.spec.components.is_none() {
			self.spec.components = Some(MurApiComponents::default());
		}
		if let Some(ref mut components) = self.spec.components {
			components.schemas.insert(name.into(), schema);
		}
		self
	}

	pub fn security_scheme(
		mut self,
		name: impl Into<String>,
		scheme: MurApiSecurityScheme,
	) -> Self {
		if self.spec.components.is_none() {
			self.spec.components = Some(MurApiComponents::default());
		}
		if let Some(ref mut components) = self.spec.components {
			components.security_schemes.insert(name.into(), scheme);
		}
		self
	}

	pub fn bearer_auth(self) -> Self {
		self.security_scheme("bearerAuth", MurApiSecurityScheme::bearer())
	}

	pub fn api_key_auth(self, header_name: impl Into<String>) -> Self {
		self.security_scheme("apiKey", MurApiSecurityScheme::api_key_header(header_name))
	}

	pub fn basic_auth(self) -> Self {
		self.security_scheme("basicAuth", MurApiSecurityScheme::basic())
	}

	pub fn security(mut self, name: impl Into<String>, scopes: Vec<String>) -> Self {
		let mut req = HashMap::new();
		req.insert(name.into(), scopes);
		self.spec.security.push(req);
		self
	}

	pub fn external_docs(mut self, url: impl Into<String>, description: impl Into<String>) -> Self {
		self.spec.external_docs = Some(MurApiExternalDocs {
			url: url.into(),
			description: Some(description.into()),
		});
		self
	}

	pub fn build(self) -> MurOpenApiSpec {
		self.spec
	}

	pub fn to_json(&self) -> Result<String, serde_json::Error> {
		self.spec.to_json()
	}

	pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
		self.spec.to_json_compact()
	}
}
