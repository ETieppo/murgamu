use crate::MurInjects;
use crate::server::router::MurRouteDefinition;
use crate::server::service::MurServiceContainer;
use std::sync::Arc;

/// Converts a value into a type-erased `Arc<dyn MurController>`.
///
/// This trait is implemented for any type that implements [`MurController`] and
/// for `Arc<dyn MurController>` itself, enabling ergonomic registration inside
/// `#[module]` controller lists.
pub trait IntoController {
	/// Wraps `self` in an `Arc` and erases the concrete type.
	fn into_controller(self) -> Arc<dyn MurController>;
}

impl<T: MurController + 'static> IntoController for T {
	fn into_controller(self) -> Arc<dyn MurController> {
		Arc::new(self)
	}
}

impl IntoController for Arc<dyn MurController> {
	fn into_controller(self) -> Arc<dyn MurController> {
		self
	}
}

/// Core trait implemented by every Murgamu controller.
///
/// Controllers group related route handlers under a common base path. The
/// `#[controller]` attribute macro generates the `MurController` implementation
/// from an annotated `impl` block automatically — you generally do not need to
/// implement this trait manually.
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
///     #[get(":id")]
///     async fn find_one(&self, #[param] id: Param<u64>) -> MurRes { /* … */ }
/// }
/// ```
pub trait MurController: Send + Sync + 'static {
	/// Returns all route definitions declared in this controller.
	///
	/// Called once during server startup to populate the router.
	fn routes(self: Arc<Self>, container: &MurServiceContainer) -> Vec<MurRouteDefinition>;

	/// Returns the base path prefix for all routes in this controller.
	///
	/// Defaults to `"/"`. The `#[controller("path")]` argument overrides this.
	fn base_path(&self) -> &str {
		"/"
	}

	/// Returns a human-readable identifier for the controller (defaults to the type name).
	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

/// Dependency-injection factory for [`MurController`] implementations.
///
/// Generated automatically by the `#[controller]` macro when the struct has
/// injected fields declared with `#[inject]`.
pub trait MurControllerFactory: MurController {
	/// Constructs the controller using dependencies from `injects` and `container`.
	fn __create_factory(injects: &MurInjects, container: &MurServiceContainer) -> Self
	where
		Self: Sized;
}

/// Marker trait combining [`MurController`] and [`Clone`].
///
/// Automatically implemented for any `T: MurController + Clone`.
pub trait MurCloneController: MurController + Clone {}
impl<T: MurController + Clone> MurCloneController for T {}
