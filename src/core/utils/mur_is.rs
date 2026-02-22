use crate::server::error::MurError;

pub struct MurIs;

impl MurIs {
	pub fn mur_validate_not_empty(value: &str, field_name: &str) -> Result<(), MurError> {
		if value.trim().is_empty() {
			Err(MurError::BadRequest(format!(
				"{} cannot be empty",
				field_name
			)))
		} else {
			Ok(())
		}
	}

	pub fn mur_validate_length(
		value: &str,
		field_name: &str,
		min: Option<usize>,
		max: Option<usize>,
	) -> Result<(), MurError> {
		let len = value.len();

		if let Some(min_len) = min
			&& len < min_len
		{
			return Err(MurError::BadRequest(format!(
				"{} must be at least {} characters",
				field_name, min_len
			)));
		}

		if let Some(max_len) = max
			&& len > max_len
		{
			return Err(MurError::BadRequest(format!(
				"{} must be at most {} characters",
				field_name, max_len
			)));
		}

		Ok(())
	}

	pub fn mur_validate_pattern(
		value: &str,
		pattern: &str,
		field_name: &str,
	) -> Result<(), MurError> {
		let re = regex::Regex::new(pattern)
			.map_err(|e| MurError::Internal(format!("Invalid regex pattern: {}", e)))?;

		if !re.is_match(value) {
			Err(MurError::BadRequest(format!(
				"{} has invalid format",
				field_name
			)))
		} else {
			Ok(())
		}
	}
}
