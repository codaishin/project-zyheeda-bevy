use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Makes serialized data better readable and smaller than serializing
/// [`Duration`] directly.
///
/// Should be used for durations, where the precision loss is negligible
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct DurationSecsF32 {
	seconds: f32,
}

impl From<DurationSecsF32> for Duration {
	fn from(DurationSecsF32 { seconds }: DurationSecsF32) -> Self {
		Duration::from_secs_f32(seconds)
	}
}

impl From<Duration> for DurationSecsF32 {
	fn from(duration: Duration) -> Self {
		DurationSecsF32 {
			seconds: duration.as_secs_f32(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_to_duration() {
		let data = DurationSecsF32 { seconds: 42.11 };

		let duration = Duration::from(data);

		assert_eq!(Duration::from_secs_f32(42.11), duration);
	}

	#[test]
	fn convert_from_duration() {
		let duration = Duration::from_secs_f32(42.11);

		let data = DurationSecsF32::from(duration);

		assert_eq!(DurationSecsF32 { seconds: 42.11 }, data);
	}
}
