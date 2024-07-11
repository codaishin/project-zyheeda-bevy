use bevy::{input::keyboard::KeyCode, state::state::States};
use common::traits::iteration::{Iter, IterFinite};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
	#[default]
	None,
	Inventory,
	ComboOverview,
}

impl IterFinite for MenuState {
	fn iterator() -> Iter<Self> {
		Iter(Some(MenuState::None))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			MenuState::None => Some(MenuState::Inventory),
			MenuState::Inventory => Some(MenuState::ComboOverview),
			MenuState::ComboOverview => None,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct NoKeySet;

impl TryFrom<MenuState> for KeyCode {
	type Error = NoKeySet;

	fn try_from(value: MenuState) -> Result<Self, Self::Error> {
		match value {
			MenuState::None => Err(NoKeySet),
			MenuState::Inventory => Ok(KeyCode::KeyI),
			MenuState::ComboOverview => Ok(KeyCode::KeyK),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_all_states() {
		assert_eq!(
			vec![
				MenuState::None,
				MenuState::Inventory,
				MenuState::ComboOverview
			],
			MenuState::iterator().collect::<Vec<_>>(),
		)
	}

	#[test]
	fn get_key_codes() {
		assert_eq!(
			vec![Err(NoKeySet), Ok(KeyCode::KeyI), Ok(KeyCode::KeyK)],
			MenuState::iterator()
				.map(KeyCode::try_from)
				.collect::<Vec<_>>()
		)
	}
}
