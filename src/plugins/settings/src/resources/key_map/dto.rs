use super::KeyMap;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::handles_custom_assets::AssetFileExtensions,
};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyMapDto<TAllKeys, TKeyCode>
where
	TAllKeys: Eq + Hash,
	TKeyCode: PartialEq,
{
	pub(crate) actions: Vec<(TAllKeys, TKeyCode)>,
}

impl<TAllActions, TInput, const N: usize> From<[(TAllActions, TInput); N]>
	for KeyMapDto<TAllActions, TInput>
where
	TAllActions: Eq + Hash,
	TInput: PartialEq,
{
	fn from(data: [(TAllActions, TInput); N]) -> Self {
		Self {
			actions: Vec::from(data),
		}
	}
}

impl AssetFileExtensions for KeyMapDto<ActionKey, UserInput> {
	fn asset_file_extensions() -> &'static [&'static str] {
		&[".keys"]
	}
}

impl From<KeyMap> for KeyMapDto<ActionKey, UserInput> {
	fn from(KeyMap(map): KeyMap) -> Self {
		Self {
			actions: Vec::from_iter(map.action_to_input),
		}
	}
}
