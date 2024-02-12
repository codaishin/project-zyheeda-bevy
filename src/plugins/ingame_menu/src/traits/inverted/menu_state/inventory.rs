use crate::{tools::MenuState, traits::inverted::Inverted};
use common::components::Inventory;

impl Inverted<Inventory> for MenuState {
	fn inverted(&self) -> Self {
		match self {
			MenuState::None => MenuState::Inventory,
			MenuState::Inventory => MenuState::None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn inventory_on() {
		let state = MenuState::None;

		assert_eq!(MenuState::Inventory, state.inverted());
	}

	#[test]
	fn inventory_off() {
		let state = MenuState::Inventory;

		assert_eq!(MenuState::None, state.inverted());
	}
}
