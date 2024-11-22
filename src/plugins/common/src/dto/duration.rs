use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct DurationDto {
	seconds: f32,
}

impl From<DurationDto> for Duration {
	fn from(DurationDto { seconds }: DurationDto) -> Self {
		Duration::from_secs_f32(seconds)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_to_duration() {
		let data = DurationDto { seconds: 42.11 };

		let duration = Duration::from(data);

		assert_eq!(Duration::from_secs_f32(42.11), duration);
	}
}
