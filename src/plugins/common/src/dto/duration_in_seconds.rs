use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A compact dto for [`Duration`].
///
/// Saturating conversion semantics when converting into [`Duration`]:
/// - NaN or negative: `Duration::ZERO`
/// - Positive infinity or overflow: `Duration::MAX`
///
/// Use only where the precision loss of `f32` is acceptable.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct DurationInSeconds {
	seconds: f32,
}

impl From<DurationInSeconds> for Duration {
	fn from(DurationInSeconds { seconds }: DurationInSeconds) -> Self {
		if seconds.is_nan() || seconds.is_sign_negative() {
			return Duration::ZERO;
		}

		Duration::try_from_secs_f32(seconds).unwrap_or(Duration::MAX)
	}
}

impl From<Duration> for DurationInSeconds {
	fn from(duration: Duration) -> Self {
		DurationInSeconds {
			seconds: duration.as_secs_f32(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_to_duration() {
		let data = DurationInSeconds { seconds: 42.11 };

		let duration = Duration::from(data);

		assert_eq!(Duration::from_secs_f32(42.11), duration);
	}

	#[test]
	fn convert_from_duration() {
		let duration = Duration::from_secs_f32(42.11);

		let data = DurationInSeconds::from(duration);

		assert_eq!(DurationInSeconds { seconds: 42.11 }, data);
	}

	#[test]
	fn round_trip_duration_max() {
		let duration = Duration::MAX;

		let data = DurationInSeconds::from(duration);

		assert_eq!(Duration::MAX, Duration::from(data));
	}

	#[test]
	fn nan_secs_to_duration() {
		let data = DurationInSeconds { seconds: f32::NAN };

		let duration = Duration::from(data);

		assert_eq!(Duration::ZERO, duration);
	}

	#[test]
	fn negative_secs_to_duration() {
		let data = DurationInSeconds { seconds: -42. };

		let duration = Duration::from(data);

		assert_eq!(Duration::ZERO, duration);
	}
}
