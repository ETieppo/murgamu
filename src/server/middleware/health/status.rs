use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MurHealthStatus {
	Up,
	Down,
	Degraded,
	#[default]
	Unknown,
}

impl MurHealthStatus {
	pub fn is_healthy(&self) -> bool {
		matches!(self, MurHealthStatus::Up | MurHealthStatus::Degraded)
	}

	pub fn http_status_code(&self) -> u16 {
		match self {
			MurHealthStatus::Up => 200,
			MurHealthStatus::Degraded => 200,
			MurHealthStatus::Down => 503,
			MurHealthStatus::Unknown => 503,
		}
	}

	pub fn combine(self, other: MurHealthStatus) -> MurHealthStatus {
		match (self, other) {
			(MurHealthStatus::Down, _) | (_, MurHealthStatus::Down) => MurHealthStatus::Down,
			(MurHealthStatus::Unknown, _) | (_, MurHealthStatus::Unknown) => {
				MurHealthStatus::Unknown
			}
			(MurHealthStatus::Degraded, _) | (_, MurHealthStatus::Degraded) => {
				MurHealthStatus::Degraded
			}
			(MurHealthStatus::Up, MurHealthStatus::Up) => MurHealthStatus::Up,
		}
	}
}

impl std::fmt::Display for MurHealthStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MurHealthStatus::Up => write!(f, "up"),
			MurHealthStatus::Down => write!(f, "down"),
			MurHealthStatus::Unknown => write!(f, "unknown"),
			MurHealthStatus::Degraded => write!(f, "degraded"),
		}
	}
}
