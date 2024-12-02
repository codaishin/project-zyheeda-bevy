use crate::components::player::Player;
use bevy::prelude::*;

#[derive(Bundle, Debug, PartialEq, Default)]
pub struct PlayerBundle {
	pub player: Player,
	pub visibility: Visibility,
	pub inherited_visibility: InheritedVisibility,
	pub view_visibility: ViewVisibility,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}
