use crate::{
	behaviors::{ItemBehavior, PlayerBehavior},
	components::{Cast, SlotKey},
	traits::from_source::FromSource,
};
use bevy::prelude::{Ray, Vec3};

fn movement(ray: Ray) -> Option<PlayerBehavior> {
	let length = ray.intersect_plane(Vec3::ZERO, Vec3::Y)?;
	Some(PlayerBehavior::MoveTo(ray.origin + ray.direction * length))
}

fn shoot(ray: Ray, cast: Cast, slot: SlotKey) -> Option<PlayerBehavior> {
	let SlotKey::Hand(side) = slot else {
		return None;
	};
	Some(PlayerBehavior::ShootGun(ray, cast, side))
}

impl FromSource<ItemBehavior, (SlotKey, Ray)> for PlayerBehavior {
	fn from_source(source: ItemBehavior, (slot, ray): (SlotKey, Ray)) -> Option<Self> {
		match (source, slot) {
			(ItemBehavior::Move, _) => movement(ray),
			(ItemBehavior::ShootGun(cast), slot) => shoot(ray, cast, slot),
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

#[cfg(test)]
mod from_item_behavior_shoot_tests {
	use super::*;
	use crate::components::{Cast, Seconds, Side};

	#[test]
	fn test_shoot_right() {
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::Z,
		};
		let cast = Cast {
			pre: Seconds(0.1),
			after: Seconds(1.4),
		};
		let shoot = ItemBehavior::ShootGun(cast);

		let shoot = PlayerBehavior::from_source(shoot, (SlotKey::Hand(Side::Right), ray));

		assert_eq!(
			Some(PlayerBehavior::ShootGun(ray, cast, Side::Right)),
			shoot
		);
	}

	#[test]
	fn test_shoot_left() {
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::Z,
		};
		let cast = Cast {
			pre: Seconds(0.1),
			after: Seconds(1.4),
		};
		let shoot = ItemBehavior::ShootGun(cast);

		let shoot = PlayerBehavior::from_source(shoot, (SlotKey::Hand(Side::Left), ray));

		assert_eq!(Some(PlayerBehavior::ShootGun(ray, cast, Side::Left)), shoot);
	}

	#[test]
	fn test_shoot_none_if_slot_not_hand() {
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::Z,
		};
		let cast = Cast {
			pre: Seconds(0.1),
			after: Seconds(1.4),
		};
		let shoot = ItemBehavior::ShootGun(cast);

		let shoot = PlayerBehavior::from_source(shoot, (SlotKey::Legs, ray));

		assert_eq!(None, shoot);
	}
}
