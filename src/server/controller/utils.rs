use super::IntoController;
use super::MurControllers;

pub fn controllers<I, C>(items: I) -> MurControllers
where
	I: IntoIterator<Item = C>,
	C: IntoController,
{
	items.into_iter().map(|c| c.into_controller()).collect()
}
