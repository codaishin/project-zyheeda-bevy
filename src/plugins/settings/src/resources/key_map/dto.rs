use super::KeyMap;
use bevy::input::keyboard::KeyCode;
use common::{tools::keys::Key, traits::handles_custom_assets::AssetFileExtensions};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyMapDto<TAllKeys, TKeyCode>
where
	TAllKeys: Eq + Hash,
	TKeyCode: PartialEq,
{
	pub(crate) keys: Vec<(TAllKeys, TKeyCode)>,
}

impl<TAllKeys, TKeyCode, const N: usize> From<[(TAllKeys, TKeyCode); N]>
	for KeyMapDto<TAllKeys, TKeyCode>
where
	TAllKeys: Eq + Hash,
	TKeyCode: PartialEq,
{
	fn from(data: [(TAllKeys, TKeyCode); N]) -> Self {
		Self {
			keys: Vec::from(data),
		}
	}
}

impl AssetFileExtensions for KeyMapDto<Key, KeyCode> {
	fn asset_file_extensions() -> &'static [&'static str] {
		&[".keys"]
	}
}

impl From<KeyMap> for KeyMapDto<Key, KeyCode> {
	fn from(KeyMap(map): KeyMap) -> Self {
		Self {
			keys: Vec::from_iter(map.key_to_key_code),
		}
	}
}
