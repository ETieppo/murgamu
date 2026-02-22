use std::any::{Any, TypeId};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};

use murgamu::{
	MurController, MurInjects, MurModule, MurRequestContext, MurRes,
	MurRouteDefinition, MurService, MurServiceContainer, MurServiceFactory, mur_json,
	serde_json,
};

// ======================================================
// ===================== SERVICES ========================
// ======================================================

pub struct SecondService;

impl SecondService {
	pub fn new() -> Self {
		Self
	}
}

impl MurService for SecondService {
	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl MurServiceFactory for SecondService {
	fn create(_injects: &MurInjects, _container: &MurServiceContainer) -> Self {
		Self::new()
	}
}

pub struct ServiceTest {
	second_service: Arc<SecondService>,
}

impl ServiceTest {
	pub fn new(second_service: Arc<SecondService>) -> Self {
		Self { second_service }
	}

	pub fn hello(&self) -> MurRes {
		let _ = &self.second_service;
		mur_json!({ "message": "hello world" })
	}
}

impl MurService for ServiceTest {
	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl MurServiceFactory for ServiceTest {
	fn create(injects: &MurInjects, container: &MurServiceContainer) -> Self {
		let second_service = container.get_required::<SecondService>();
		Self::new(second_service)
	}
}

// ======================================================
// ==================== CONTROLLER ======================
// ======================================================

pub struct ControllerTest {
	service_test: Arc<ServiceTest>,
}

impl ControllerTest {
	pub fn new(service_test: Arc<ServiceTest>) -> Self {
		Self { service_test }
	}

	pub async fn helo(&self) -> MurRes {
		self.service_test.hello()
	}
}

impl MurController for ControllerTest {
	fn routes(self: Arc<Self>, _container: Arc<MurServiceContainer>) -> Vec<MurRouteDefinition> {
		let mut routes = Vec::new();
		let controller = self.clone();

		// GET /test/helo
		let handler = {
			let controller_clone = controller.clone();

			Arc::new(
				move |_ctx: MurRequestContext| -> Pin<Box<dyn Future<Output = MurRes> + Send>> {
					let controller = controller_clone.clone();
					Box::pin(async move { controller.helo().await })
				},
			)
		};

		routes.push(MurRouteDefinition {
			method: "GET".to_string(),
			path: "/test/helo".to_string(),
			handler,
			is_public: true,
			allowed_roles: vec![],
		});

		routes
	}

	fn base_path(&self) -> &str {
		"/test"
	}
}

// ======================================================
// ====================== MODULE ========================
// ======================================================

pub struct ModuleTest {
	second_service: OnceLock<Arc<SecondService>>,
	service_test: OnceLock<Arc<ServiceTest>>,
}

impl ModuleTest {
	pub fn new() -> Self {
		Self {
			second_service: OnceLock::new(),
			service_test: OnceLock::new(),
		}
	}

	fn __mur_service_second_service(
		&self,
		injects: &MurInjects,
		container: &MurServiceContainer,
	) -> Arc<SecondService> {
		self.second_service
			.get_or_init(|| {
				Arc::new(<SecondService as MurServiceFactory>::create(
					injects, container,
				))
			})
			.clone()
	}

	fn __mur_service_service_test(
		&self,
		injects: &MurInjects,
		container: &MurServiceContainer,
	) -> Arc<ServiceTest> {
		self.service_test
			.get_or_init(|| {
				let second = self.__mur_service_second_service(injects, container);
				Arc::new(ServiceTest::new(second))
			})
			.clone()
	}
}

impl Default for ModuleTest {
	fn default() -> Self {
		Self::new()
	}
}

impl MurModule for ModuleTest {
	fn name(&self) -> &str {
		"ModuleTest"
	}

	fn controllers_with_injects(&self, _injects: &MurInjects) -> Vec<Arc<dyn MurController>> {
		vec![]
	}

	fn controllers(&self) -> Vec<Arc<dyn MurController>> {
		self.controllers_with_injects(&MurInjects::default())
	}

	fn imports(&self) -> Vec<Arc<dyn MurModule>> {
		vec![]
	}

	fn exports(&self) -> Vec<TypeId> {
		vec![TypeId::of::<SecondService>(), TypeId::of::<ServiceTest>()]
	}

	fn services_with_injects(
		&self,
		injects: &MurInjects,
		container: &MurServiceContainer,
	) -> Vec<(TypeId, Arc<dyn MurService>)> {
		vec![
			(
				TypeId::of::<SecondService>(),
				self.__mur_service_second_service(injects, container) as Arc<dyn MurService>,
			),
			(
				TypeId::of::<ServiceTest>(),
				self.__mur_service_service_test(injects, container) as Arc<dyn MurService>,
			),
		]
	}

	fn services(&self) -> Vec<(TypeId, Arc<dyn MurService>)> {
		vec![]
	}
}
