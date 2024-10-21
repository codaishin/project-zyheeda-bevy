pub mod item;
pub mod traits;

use bevy::prelude::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
	fn build(&self, _: &mut App) {}
}
