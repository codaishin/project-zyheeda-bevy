use crate::{
	behaviors::{ItemBehavior, PlayerBehavior},
	components::SlotKey,
	traits::from_source::FromSource,
};
use bevy::prelude::{Ray, Vec3};

fn movement(ray: Ray) -> Option<PlayerBehavior> {
	let length = ray.intersect_plane(Vec3::ZERO, Vec3::Y)?;
	Some(PlayerBehavior::MoveTo(ray.origin + ray.direction * length))
}

impl FromSource<ItemBehavior, (SlotKey, Ray)> for PlayerBehavior {
	fn from_source(source: ItemBehavior, (_, ray): (SlotKey, Ray)) -> Option<Self> {
		match source {
			ItemBehavior::Move => movement(ray),
		}
	}
}

#[cfg(test)]
mod from_item_behavior_move_tests {
	use super::*;
	use bevy::prelude::Vec3;

	#[test]
	fn move_to_zero() {
		let movement_partial = ItemBehavior::Move;
		let movement = PlayerBehavior::from_source(
			movement_partial,
			(
				SlotKey::Legs,
				Ray {
					origin: Vec3::Y,
					direction: Vec3::NEG_Y,
				},
			),
		);
		assert_eq!(Some(PlayerBehavior::MoveTo(Vec3::ZERO)), movement);
	}

	#[test]
	fn move_to_offset() {
		let movement_partial = ItemBehavior::Move;
		let movement = PlayerBehavior::from_source(
			movement_partial,
			(
				SlotKey::Legs,
				Ray {
					origin: Vec3::ONE,
					direction: Vec3::NEG_Y,
				},
			),
		);
		assert_eq!(
			Some(PlayerBehavior::MoveTo(Vec3::new(1., 0., 1.))),
			movement
		);
	}

	#[test]
	fn move_to_offset_2() {
		let movement_partial = ItemBehavior::Move;
		let movement = PlayerBehavior::from_source(
			movement_partial,
			(
				SlotKey::Legs,
				Ray {
					origin: Vec3::ONE * 2.,
					direction: Vec3::NEG_Y,
				},
			),
		);
		assert_eq!(
			Some(PlayerBehavior::MoveTo(Vec3::new(2., 0., 2.))),
			movement
		);
	}
}
