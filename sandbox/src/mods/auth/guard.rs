use murgamu::prelude::*;

#[guard]
pub struct GlobalGuard {}

impl GlobalGuard {
	pub async fn can_activate(&self, ctx: &MurRequestContext) -> bool {
		true
	}
}
