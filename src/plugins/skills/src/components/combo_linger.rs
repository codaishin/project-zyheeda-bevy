use crate::traits::{Flush, IsLingering};
use bevy::ecs::component::Component;
use std::time::Duration;

#[derive(Component)]
pub(crate) struct ComboLinger {
	max_duration: Duration,
	duration: Duration,
}

impl ComboLinger {
	pub fn new(duration: Duration) -> Self {
		Self {
			max_duration: duration,
			duration: Duration::ZERO,
		}
	}
}

impl IsLingering for ComboLinger {
	fn is_lingering(&mut self, delta: Duration) -> bool {
		self.duration += delta;
		self.duration <= self.max_duration
	}
}

impl Flush for ComboLinger {
	fn flush(&mut self) {
		self.duration = Duration::ZERO;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn linger_when_delta_smaller_than_duration() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));

		assert!(linger.is_lingering(Duration::from_secs(1)));
	}

	#[test]
	fn linger_when_delta_greater_than_duration() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));

		assert!(!linger.is_lingering(Duration::from_secs(4)));
	}

	#[test]
	fn linger_when_delta_equal_to_duration() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));

		assert!(linger.is_lingering(Duration::from_secs(3)));
	}

	#[test]
	fn linger_step_by_step_progression() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));
		let lingers = [
			linger.is_lingering(Duration::from_secs(1)),
			linger.is_lingering(Duration::from_secs(1)),
			linger.is_lingering(Duration::from_secs(1)),
			linger.is_lingering(Duration::from_secs(1)),
		];

		assert_eq!([true, true, true, false], lingers);
	}

	#[test]
	fn flush_resets_measurement() {
		let mut linger = ComboLinger::new(Duration::from_secs(1));
		linger.is_lingering(Duration::from_secs(1));
		linger.is_lingering(Duration::from_secs(1));
		linger.flush();

		let lingers = [
			linger.is_lingering(Duration::from_secs(1)),
			linger.is_lingering(Duration::from_secs(1)),
		];

		assert_eq!([true, false], lingers);
	}
}
