use crate::{
	components::{PlayerSkills, SideUnset, Track},
	skill::{Active, Skill, SkillState},
	traits::state_duration::StateDuration,
};
use std::time::Duration;

impl StateDuration<SkillState> for Track<Skill<PlayerSkills<SideUnset>, Active>> {
	fn elapsed_mut(&mut self) -> &mut Duration {
		&mut self.elapsed
	}

	fn get_state_duration(&self, key: SkillState) -> Duration {
		match key {
			SkillState::PreTransition => self.value.data.pre_transition,
			SkillState::PreCast => self.value.cast.pre,
			SkillState::Active => self.value.cast.active,
			SkillState::AfterCast => self.value.cast.after,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skill::{Cast, SkillState};
	use bevy::utils::default;

	#[test]
	fn get_phasing_times() {
		let track = Track::new(Skill {
			data: Active::default(),
			cast: Cast {
				pre: Duration::from_millis(1),
				active: Duration::from_millis(2),
				after: Duration::from_millis(3),
			},
			..default()
		});

		assert_eq!(
			[
				(Duration::from_millis(1), SkillState::PreCast),
				(Duration::from_millis(2), SkillState::Active),
				(Duration::from_millis(3), SkillState::AfterCast),
			],
			[
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			]
			.map(|state| (track.get_state_duration(state), state))
		)
	}

	#[test]
	fn get_duration() {
		let mut track = Track::new(Skill {
			data: Active::default(),
			..default()
		});

		*track.elapsed_mut() = Duration::from_secs(42);

		assert_eq!(Duration::from_secs(42), track.elapsed);
	}
}
