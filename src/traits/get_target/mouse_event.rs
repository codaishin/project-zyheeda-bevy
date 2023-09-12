use super::GetTarget;
use crate::MouseEvent;
use bevy::prelude::*;

impl GetTarget for MouseEvent {
	fn get_target(&self) -> Option<Vec3> {
		self.world_position
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		events::{MouseEvent, Plane},
		traits::get_target::GetTarget,
	};
	use bevy::prelude::Vec3;

	#[test]
	fn get_world_position_none() {
		let event = MouseEvent {
			collision_plane: Plane {
				origin: Vec3::ZERO,
				normal: Vec3::Y,
			},
			world_position: None,
		};

		assert_eq!(None, event.get_target());
	}

	#[test]
	fn get_world_position_some() {
		let event = MouseEvent {
			collision_plane: Plane {
				origin: Vec3::ZERO,
				normal: Vec3::Y,
			},
			world_position: Some(Vec3::ONE),
		};

		assert_eq!(Some(Vec3::ONE), event.get_target());
	}
}
