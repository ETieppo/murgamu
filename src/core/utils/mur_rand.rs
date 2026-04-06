use super::MurTime;

pub struct MurRand;

impl MurRand {
	pub fn mur_gen_id() -> String {
		let timestamp = MurTime::timestamp_ms();
		let random: u32 = Self::rand_u32();
		format!("{:x}{:08x}", timestamp, random)
	}

	fn rand_u32() -> u32 {
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hash, Hasher};

		let mut hasher = DefaultHasher::new();
		std::time::Instant::now().hash(&mut hasher);
		std::thread::current().id().hash(&mut hasher);
		hasher.finish() as u32
	}
}
