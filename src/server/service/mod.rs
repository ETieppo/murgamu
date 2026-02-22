mod builder;
mod container;
mod contract;
mod injects;

use std::sync::Arc;

pub use builder::MurServiceContainerBuilder;
pub use container::MurServiceContainer;
pub use contract::MurInjectable;
pub use contract::MurService;
pub use contract::MurServiceFactory;
pub use injects::MurInjects;

pub type MurServices = Vec<(std::any::TypeId, Arc<dyn MurService>)>;
