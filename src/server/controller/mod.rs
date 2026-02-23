mod contract;
mod utils;

use std::sync::Arc;

pub use contract::IntoController;
pub use contract::MurCloneController;
pub use contract::MurController;
pub use contract::MurControllerFactory;
pub use utils::controllers;

pub type MurControllers = Vec<Arc<dyn MurController>>;
