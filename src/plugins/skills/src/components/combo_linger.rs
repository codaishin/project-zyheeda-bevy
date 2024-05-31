use crate::traits::{Flush, IsLingering};
use bevy::ecs::component::Component;
use common::traits::update_cumulative::CumulativeUpdate;
use std::time::Duration;

#[derive(Component)]
pub struct ComboLinger {
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
	fn is_lingering(&self) -> bool {
		self.duration <= self.max_duration
	}
}

impl CumulativeUpdate<Duration> for ComboLinger {
	fn update_cumulative(&mut self, delta: Duration) {
		self.duration += delta;
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
	fn linger_when_not_updated() {
		let linger = ComboLinger::new(Duration::from_secs(3));

		assert!(linger.is_lingering());
	}

	#[test]
	fn linger_when_updated_greater_than_duration() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));

		linger.update_cumulative(Duration::from_secs(4));

		assert!(!linger.is_lingering());
	}

	#[test]
	fn linger_when_updated_equal_to_duration() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));

		linger.update_cumulative(Duration::from_secs(3));
		assert!(linger.is_lingering());
	}

	#[test]
	fn linger_step_by_step_progression() {
		let mut linger = ComboLinger::new(Duration::from_secs(3));
		let lingers = [
			{
				linger.update_cumulative(Duration::from_secs(1));
				linger.is_lingering()
			},
			{
				linger.update_cumulative(Duration::from_secs(1));
				linger.is_lingering()
			},
			{
				linger.update_cumulative(Duration::from_secs(1));
				linger.is_lingering()
			},
			{
				linger.update_cumulative(Duration::from_secs(1));
				linger.is_lingering()
			},
		];

		assert_eq!([true, true, true, false], lingers);
	}

	#[test]
	fn flush_resets_measurement() {
		let mut linger = ComboLinger::new(Duration::from_secs(1));
		linger.update_cumulative(Duration::from_secs(1));
		linger.is_lingering();
		linger.update_cumulative(Duration::from_secs(1));
		linger.is_lingering();
		linger.flush();

		let lingers = [
			{
				linger.update_cumulative(Duration::from_secs(1));
				linger.is_lingering()
			},
			{
				linger.update_cumulative(Duration::from_secs(1));
				linger.is_lingering()
			},
		];

		assert_eq!([true, false], lingers);
	}
}
