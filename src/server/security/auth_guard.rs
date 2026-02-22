use murgamu::prelude::*;
use std::collections::HashSet;

pub trait HasRole {
	fn role_name(&self) -> &'static str;
}

#[guard]
pub struct AuthGuard<R: HasRole + Send + Sync + 'static> {
	_marker: std::marker::PhantomData<R>,
}

impl<R: HasRole + Send + Sync + 'static> AuthGuard<R> {
	pub fn new() -> Self {
		Self {
			_marker: std::marker::PhantomData,
		}
	}

	pub async fn check(&self, ctx: &MurRequestContext) -> bool {
		// 1. Se a rota é pública, permite
		if ctx.is_public_route() {
			return true;
		}

		// 2. Verificar se o usuário está autenticado
		let token = match ctx.bearer_token() {
			Some(t) => t,
			None => return false,
		};

		// 3. Validar o token e extrair as roles do usuário
		let user_roles = match self.validate_token_and_get_roles(token).await {
			Some(roles) => roles,
			None => return false,
		};

		// 4. Se a rota tem roles específicas, verificar
		if let Some(allowed_roles) = ctx.allowed_roles() {
			if allowed_roles.is_empty() {
				return true; // Autenticado é suficiente
			}
			// Verificar se o usuário tem pelo menos uma das roles permitidas
			return user_roles.iter().any(|r| allowed_roles.contains(r));
		}

		true // Autenticado e sem restrição de roles
	}

	async fn validate_token_and_get_roles(&self, token: &str) -> Option<HashSet<String>> {
		// Implementar validação JWT aqui
		// Retornar as roles do usuário
		todo!()
	}
}
