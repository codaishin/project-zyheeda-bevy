use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct DurationData {
	seconds: f32,
}

impl From<DurationData> for Duration {
	fn from(DurationData { seconds }: DurationData) -> Self {
		Duration::from_secs_f32(seconds)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_to_duration() {
		let data = DurationData { seconds: 42.11 };

		let duration = Duration::from(data);

		assert_eq!(Duration::from_secs_f32(42.11), duration);
	}
}
