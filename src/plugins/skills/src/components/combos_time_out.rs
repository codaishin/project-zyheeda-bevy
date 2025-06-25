pub(crate) mod dto;

use crate::{
	CombosTimeOutDto,
	traits::{Flush, is_timed_out::IsTimedOut},
};
use bevy::ecs::component::Component;
use common::traits::update_cumulative::CumulativeUpdate;
use macros::SavableComponent;
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[savable_component(dto = CombosTimeOutDto)]
pub struct CombosTimeOut {
	max_duration: Duration,
	duration: Duration,
}

impl CombosTimeOut {
	pub fn after(max_duration: Duration) -> Self {
		Self {
			max_duration,
			duration: Duration::ZERO,
		}
	}
}

impl IsTimedOut for CombosTimeOut {
	fn is_timed_out(&self) -> bool {
		self.duration > self.max_duration
	}
}

impl CumulativeUpdate<Duration> for CombosTimeOut {
	fn update_cumulative(&mut self, delta: Duration) {
		self.duration += delta;
	}
}

impl Flush for CombosTimeOut {
	fn flush(&mut self) {
		self.duration = Duration::ZERO;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn not_timed_out_when_not_updated() {
		let timeout = CombosTimeOut::after(Duration::from_secs(3));

		assert!(!timeout.is_timed_out());
	}

	#[test]
	fn timed_out_when_updated_greater_than_duration() {
		let mut timeout = CombosTimeOut::after(Duration::from_secs(3));

		timeout.update_cumulative(Duration::from_secs(4));

		assert!(timeout.is_timed_out());
	}

	#[test]
	fn not_timed_out_when_updated_equal_to_duration() {
		let mut timeout = CombosTimeOut::after(Duration::from_secs(3));

		timeout.update_cumulative(Duration::from_secs(3));
		assert!(!timeout.is_timed_out());
	}

	#[test]
	fn step_by_step_timeout_progression() {
		let mut timeout = CombosTimeOut::after(Duration::from_secs(3));
		let timeouts = [
			{
				timeout.update_cumulative(Duration::from_secs(1));
				timeout.is_timed_out()
			},
			{
				timeout.update_cumulative(Duration::from_secs(1));
				timeout.is_timed_out()
			},
			{
				timeout.update_cumulative(Duration::from_secs(1));
				timeout.is_timed_out()
			},
			{
				timeout.update_cumulative(Duration::from_secs(1));
				timeout.is_timed_out()
			},
		];

		assert_eq!([false, false, false, true], timeouts);
	}

	#[test]
	fn flush_resets_measurement() {
		let mut timeout = CombosTimeOut::after(Duration::from_secs(1));
		timeout.update_cumulative(Duration::from_secs(1));
		timeout.is_timed_out();
		timeout.update_cumulative(Duration::from_secs(1));
		timeout.is_timed_out();
		timeout.flush();

		let lingers = [
			{
				timeout.update_cumulative(Duration::from_secs(1));
				timeout.is_timed_out()
			},
			{
				timeout.update_cumulative(Duration::from_secs(1));
				timeout.is_timed_out()
			},
		];

		assert_eq!([false, true], lingers);
	}
}
