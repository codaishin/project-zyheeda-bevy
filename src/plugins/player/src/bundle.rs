use crate::components::player::Player;
use bevy::prelude::*;
use common::components::Health;

#[derive(Bundle, Debug, PartialEq)]
pub struct PlayerBundle {
	pub player: Player,
	pub health: Health,
	pub visibility: Visibility,
	pub inherited_visibility: InheritedVisibility,
	pub view_visibility: ViewVisibility,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}

impl Default for PlayerBundle {
	fn default() -> Self {
		Self {
			player: Default::default(),
			health: Health::new(100.),
			visibility: Default::default(),
			inherited_visibility: Default::default(),
			view_visibility: Default::default(),
			transform: Default::default(),
			global_transform: Default::default(),
		}
	}
}
