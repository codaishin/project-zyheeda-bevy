use super::{CastType, CastUpdate, State};
use crate::components::{Active, Skill};
use std::time::Duration;

fn new_or_active(skill: &Skill<Active>) -> State {
	if skill.cast.pre == Duration::ZERO {
		State::Activate
	} else {
		State::New
	}
}

fn full_cast(skill: &Skill<Active>) -> Duration {
	if skill.data.ignore_after_cast {
		skill.cast.pre
	} else {
		skill.cast.pre + skill.cast.after
	}
}

impl CastUpdate for Skill<Active> {
	fn update(&mut self, delta: Duration) -> State {
		let old_duration = self.data.duration;

		self.data.duration += delta;

		match (old_duration, self.data.duration) {
			(Duration::ZERO, _) => new_or_active(self),
			(_, new_duration) if new_duration < self.cast.pre => State::Casting(CastType::Pre),
			(old_duration, _) if old_duration < self.cast.pre => State::Activate,
			(_, new_duration) if new_duration < full_cast(self) => State::Casting(CastType::After),
			_ => State::Done,
		}
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

	#[test]
	fn no_double_activate() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(0),
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		let states = [
			skill.update(Duration::from_millis(50)),
			skill.update(Duration::from_millis(50)),
			skill.update(Duration::from_millis(50)),
		];

		assert_eq!(
			[State::New, State::Activate, State::Casting(CastType::After)],
			states
		);
	}

	#[test]
	fn ignore_after_cast() {
		let mut skill = Skill {
			data: Active {
				duration: Duration::from_millis(0),
				ignore_after_cast: true,
				..default()
			},
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		};

		let states = [
			skill.update(Duration::from_millis(50)),
			skill.update(Duration::from_millis(50)),
			skill.update(Duration::from_millis(50)),
		];

		assert_eq!([State::New, State::Activate, State::Done], states);
	}
}
