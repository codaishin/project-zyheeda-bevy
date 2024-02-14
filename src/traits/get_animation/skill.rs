use super::GetAnimation;
use common::{
	components::{Animate, Side, SideUnset, SlotKey, Track},
	skill::{Active, PlayerSkills, Skill},
};

impl GetAnimation<PlayerSkills<Side>> for Track<Skill<PlayerSkills<SideUnset>, Active>> {
	fn animate(&self) -> Animate<PlayerSkills<Side>> {
		match (self.value.animate, self.value.data.slot_key) {
			(PlayerSkills::Idle, _) => Animate::Repeat(PlayerSkills::Idle),
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
	use bevy::prelude::default;
	use common::components::Handed;

	#[test]
	fn get_simple_repeat_animations() {
		let animates: [PlayerSkills<SideUnset>; 1] = [PlayerSkills::Idle];
		let tracks = animates.map(|animate| {
			Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
				animate,
				..default()
			})
		});

		assert_eq!(
			[Animate::<PlayerSkills<Side>>::Repeat(PlayerSkills::Idle)],
			tracks.map(|track| track.animate())
		);
	}

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
				animate,
				..default()
			})
		});
		let off_tracks = animates.map(|animate| {
			Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
				data: Active {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				animate,
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
			animate,
			..default()
		});
		let off_track = Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
			data: Active {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			animate,
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
