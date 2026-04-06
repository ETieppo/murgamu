use murgamu::prelude::*;

#[pipe(i32)]
pub struct JwtUserExtractionPipe;

impl JwtUserExtractionPipe {
	fn transform(&self, _ctx: MurRequestContext, input: i32) -> Result<i32, MurError> {
		Ok(input + 1)
	}
}
