use super::GetAnimation;
use crate::{
	components::{SideUnset, SlotKey, Track},
	skill::{Active, PlayerSkills, Skill},
};
use common::components::{Animate, Side};

impl GetAnimation<PlayerSkills<Side>> for Track<Skill<PlayerSkills<SideUnset>, Active>> {
	fn animate(&self) -> Animate<PlayerSkills<Side>> {
		let Some(animate) = self.value.animate else {
			return Animate::None;
		};
		match (animate, self.value.data.slot_key) {
			(PlayerSkills::Shoot(dual_or_single), SlotKey::Hand(side)) => {
				Animate::Repeat(PlayerSkills::Shoot(dual_or_single.on(side)))
			}
			(PlayerSkills::SwordStrike(_), SlotKey::Hand(side)) => {
				Animate::Replay(PlayerSkills::SwordStrike(side))
			}
			_ => Animate::None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Handed;
	use bevy::prelude::default;

	#[test]
	fn get_shoot_animations() {
		let animates = [
			PlayerSkills::Shoot(Handed::Single(SideUnset)),
			PlayerSkills::Shoot(Handed::Dual(SideUnset)),
		];
		let main_tracks = animates.map(|animate| {
			Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
				data: Active {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				animate: Some(animate),
				..default()
			})
		});
		let off_tracks = animates.map(|animate| {
			Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
				data: Active {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				animate: Some(animate),
				..default()
			})
		});

		assert_eq!(
			(
				[
					Animate::Repeat(PlayerSkills::Shoot(Handed::Single(Side::Main))),
					Animate::Repeat(PlayerSkills::Shoot(Handed::Dual(Side::Main)))
				],
				[
					Animate::Repeat(PlayerSkills::Shoot(Handed::Single(Side::Off))),
					Animate::Repeat(PlayerSkills::Shoot(Handed::Dual(Side::Off)))
				],
			),
			(
				main_tracks.map(|track| track.animate()),
				off_tracks.map(|track| track.animate())
			)
		)
	}

	#[test]
	fn get_sword_strike_animations() {
		let animate = PlayerSkills::SwordStrike(SideUnset);
		let main_track = Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
			data: Active {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			animate: Some(animate),
			..default()
		});
		let off_track = Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
			data: Active {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			animate: Some(animate),
			..default()
		});

		assert_eq!(
			[
				Animate::Replay(PlayerSkills::SwordStrike(Side::Main)),
				Animate::Replay(PlayerSkills::SwordStrike(Side::Off))
			],
			[main_track.animate(), off_track.animate()]
		)
	}
}
