use crate::{
	components::{Active, Skill, Track},
	traits::cast_update::{AgeType, CastType, CastUpdate, State},
};
use std::time::Duration;

impl CastUpdate for Track<Skill<Active>> {
	fn update(&mut self, delta: Duration) -> State {
		let skill = &self.current;
		let old_duration = self.duration;

		self.duration += delta;

		match (old_duration, self.duration) {
			(Duration::ZERO, _) => new_or_active(skill),
			(_, new_duration) if new_duration < skill.cast.pre => State::Casting(CastType::Pre),
			(old_duration, _) if old_duration < skill.cast.pre => State::Active(AgeType::Old),
			(_, new_duration) if new_duration < full_cast(skill) => State::Casting(CastType::After),
			_ => State::Done,
		}
	}
}

fn new_or_active(skill: &Skill<Active>) -> State {
	if skill.cast.pre == Duration::ZERO {
		State::Active(AgeType::New)
	} else {
		State::New
	}
}

fn full_cast(skill: &Skill<Active>) -> Duration {
	skill.cast.pre + skill.cast.after
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Cast;
	use bevy::utils::default;

	#[test]
	fn update_state_with_delta() {
		let mut track = Track::new(Skill {
			data: Active { ..default() },
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});

		let state = track.update(Duration::from_millis(55));

		assert_eq!(
			(State::New, Duration::from_millis(55)),
			(state, track.duration)
		);
	}

	#[test]
	fn update_state_with_delta_multiple_times() {
		let mut track = Track::new(Skill {
			data: Active { ..default() },
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});

		let states = [
			track.update(Duration::from_millis(55)),
			track.update(Duration::from_millis(10)),
		];

		assert_eq!(
			(
				[State::New, State::Casting(CastType::Pre)],
				Duration::from_millis(65)
			),
			(states, track.duration)
		);
	}

	#[test]
	fn update_state_after_activate() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(1);

		assert_eq!(
			State::Active(AgeType::Old),
			track.update(Duration::from_millis(99))
		);
	}

	#[test]
	fn update_state_activate() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(1);

		assert_eq!(
			State::Active(AgeType::Old),
			track.update(Duration::from_millis(100))
		);
	}

	#[test]
	fn update_state_after_cast() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(101);

		assert_eq!(
			State::Casting(CastType::After),
			track.update(Duration::from_millis(90))
		);
	}

	#[test]
	fn update_state_done() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(101);

		assert_eq!(State::Done, track.update(Duration::from_millis(100)));
	}

	#[test]
	fn zero_cast_time() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(0),
				after: Duration::from_millis(0),
			},
			..default()
		});
		track.duration = Duration::from_millis(0);

		let states = [
			track.update(Duration::from_millis(1)),
			track.update(Duration::from_millis(1)),
		];

		assert_eq!(
			(
				[State::Active(AgeType::New), State::Done],
				Duration::from_millis(2)
			),
			(states, track.duration)
		);
	}

	#[test]
	fn no_double_activate() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::ZERO;

		let states = [
			track.update(Duration::from_millis(50)),
			track.update(Duration::from_millis(50)),
			track.update(Duration::from_millis(50)),
		];

		assert_eq!(
			[
				State::New,
				State::Active(AgeType::Old),
				State::Casting(CastType::After)
			],
			states
		);
	}
}
