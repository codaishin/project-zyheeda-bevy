use super::KeyMap;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{handles_custom_assets::AssetFileExtensions, iteration::IterFinite},
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
			actions: ActionKey::iterator()
				.filter_map(|action| Some((action, *map.action_to_input.get(&action)?)))
				.collect::<Vec<_>>(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::key_map::KeyMapInternal;
	use bevy::input::keyboard::KeyCode;
	use common::{tools::action_key::camera_key::CameraKey, traits::handles_settings::UpdateKey};
	use testing::repeat_scope;

	#[test]
	fn convert_to_dto_with_key_standard_ordering() {
		repeat_scope!(10, {
			let map = KeyMap(KeyMapInternal::default());

			let dto = KeyMapDto::from(map);

			assert_eq!(
				ActionKey::iterator()
					.map(|action| (action, UserInput::from(action)))
					.collect::<Vec<_>>(),
				dto.actions,
			);
		});
	}

	#[test]
	fn convert_to_dto_with_key_standard_ordering_with_overrides() {
		repeat_scope!(10, {
			let mut map = KeyMap(KeyMapInternal::default());
			map.update_key(
				ActionKey::Camera(CameraKey::Rotate),
				UserInput::KeyCode(KeyCode::F35),
			);

			let dto = KeyMapDto::from(map);

			assert_eq!(
				ActionKey::iterator()
					.map(|action| (
						action,
						if action != ActionKey::Camera(CameraKey::Rotate) {
							UserInput::from(action)
						} else {
							UserInput::KeyCode(KeyCode::F35)
						}
					))
					.collect::<Vec<_>>(),
				dto.actions,
			);
		});
	}
}
