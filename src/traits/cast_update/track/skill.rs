use crate::{
	components::Track,
	skill::{Active, Skill},
	traits::cast_update::{AgeType, CastType, CastUpdate, State},
};
use std::time::Duration;

impl<TAnimationKey> CastUpdate for Track<Skill<TAnimationKey, Active>> {
	fn update(&mut self, delta: Duration) -> State {
		let skill = &self.value;
		let old = self.duration;

		self.duration += delta;

		match (old, self.duration) {
			(Duration::ZERO, _) => new_or_activate(skill),
			(_, new) if new < pre_cast(skill) => State::Casting(CastType::Pre),
			(old, _) if old < pre_cast(skill) => State::Activate(AgeType::Old),
			(_, new) if new < active_cast(skill) => State::Active,
			(_, new) if new < after_cast(skill) => State::Casting(CastType::After),
			_ => State::Done,
		}
	}
}

fn new_or_activate<TAnimationKey>(skill: &Skill<TAnimationKey, Active>) -> State {
	if skill.cast.pre == Duration::ZERO {
		State::Activate(AgeType::New)
	} else {
		State::New
	}
}

fn pre_cast<TAnimationKey>(skill: &Skill<TAnimationKey, Active>) -> Duration {
	skill.cast.pre
}

fn active_cast<TAnimationKey>(skill: &Skill<TAnimationKey, Active>) -> Duration {
	skill.cast.pre + skill.cast.active
}

fn after_cast<TAnimationKey>(skill: &Skill<TAnimationKey, Active>) -> Duration {
	skill.cast.pre + skill.cast.active + skill.cast.after
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skill::Cast;
	use bevy::utils::default;

	#[test]
	fn update_state_with_delta() {
		let mut track = Track::new(Skill {
			data: Active { ..default() },
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::from_millis(100),
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
				active: Duration::from_millis(100),
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
				active: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(1);

		assert_eq!(
			State::Activate(AgeType::Old),
			track.update(Duration::from_millis(99))
		);
	}

	#[test]
	fn update_state_activate() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(1);

		assert_eq!(
			State::Activate(AgeType::Old),
			track.update(Duration::from_millis(100))
		);
	}

	#[test]
	fn update_state_active() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(101);

		assert_eq!(State::Active, track.update(Duration::from_millis(90)));
	}

	#[test]
	fn update_state_after_cast() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(201);

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
				active: Duration::from_millis(100),
				after: Duration::from_millis(100),
			},
			..default()
		});
		track.duration = Duration::from_millis(201);

		assert_eq!(State::Done, track.update(Duration::from_millis(100)));
	}

	#[test]
	fn zero_cast_time() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::ZERO,
				active: Duration::ZERO,
				after: Duration::ZERO,
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
				[State::Activate(AgeType::New), State::Done],
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
				active: Duration::from_millis(100),
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
			[State::New, State::Activate(AgeType::Old), State::Active],
			states
		);
	}
}
