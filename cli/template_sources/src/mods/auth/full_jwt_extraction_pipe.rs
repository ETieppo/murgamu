use murgamu::prelude::*;

pub struct JwtUserExtractionPipe;

#[pipe]
impl JwtUserExtractionPipe {
	fn new() -> Self {
		Self
	}

	fn transform(&self, _ctx: MurRequestContext, input: i32) -> Result<i32, MurError> {
		Ok(input + 1)
	}
}
