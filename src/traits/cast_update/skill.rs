use super::{CastType, CastUpdate, State};
use crate::components::{Active, Skill};
use std::time::Duration;

impl CastUpdate for Skill<Active> {
	fn update(&mut self, delta: Duration) -> State {
		let old_duration = self.data.duration;

		self.data.duration += delta;

		if old_duration == Duration::ZERO && self.cast.pre != Duration::ZERO {
			return State::New;
		}

		if self.data.duration < self.cast.pre {
			return State::Casting(CastType::Pre);
		}

		if old_duration <= self.cast.pre {
			return State::Activate;
		}

		if self.data.duration < self.cast.pre + self.cast.after {
			return State::Casting(CastType::After);
		}

		State::Done
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Cast;
	use bevy::utils::default;

	#[test]
	fn update_state_with_delta() {
		let mut skill = Skill {
			data: Active { ..default() },
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		let state = skill.update(Duration::from_millis(55));

		assert_eq!(
			(State::New, Duration::from_millis(55)),
			(state, skill.data.duration)
		);
	}

	#[test]
	fn update_state_with_delta_multiple_times() {
		let mut skill = Skill {
			data: Active { ..default() },
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		let states = [
			skill.update(Duration::from_millis(55)),
			skill.update(Duration::from_millis(10)),
		];

		assert_eq!(
			(
				[State::New, State::Casting(CastType::Pre)],
				Duration::from_millis(65)
			),
			(states, skill.data.duration)
		);
	}

	#[test]
	fn update_state_after_activate() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(1),
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		assert_eq!(State::Activate, skill.update(Duration::from_millis(99)));
	}

	#[test]
	fn update_state_activate() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(1),
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		assert_eq!(State::Activate, skill.update(Duration::from_millis(100)));
	}

	#[test]
	fn update_state_after_cast() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(101),
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		assert_eq!(
			State::Casting(CastType::After),
			skill.update(Duration::from_millis(90))
		);
	}

	#[test]
	fn update_state_done() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(101),
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		assert_eq!(State::Done, skill.update(Duration::from_millis(100)));
	}

	#[test]
	fn zero_cast_time() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(0),
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(0),
				after: Duration::from_millis(0),
			},
			..default()
		};

		let states = [
			skill.update(Duration::from_millis(1)),
			skill.update(Duration::from_millis(1)),
		];

		assert_eq!(
			([State::Activate, State::Done], Duration::from_millis(2)),
			(states, skill.data.duration)
		);
	}
}
