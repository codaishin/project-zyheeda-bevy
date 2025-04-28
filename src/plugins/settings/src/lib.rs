pub mod resources;

use bevy::prelude::*;
use common::{tools::keys::Key, traits::handles_settings::HandlesSettings};
use resources::key_map::KeyMap;

#[derive(Debug, PartialEq)]
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<KeyMap>();
	}
}

impl HandlesSettings for SettingsPlugin {
	type TKeyMap<TKey>
		= KeyMap
	where
		TKey: TryFrom<Key> + Copy,
		KeyCode: From<TKey> + PartialEq;
}
