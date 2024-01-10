use crate::plugins::ingame_menu::{
	components::InventoryPanel,
	tools::PanelState,
	traits::get::Get,
};

impl Get<(), PanelState> for InventoryPanel {
	fn get(&self, _: ()) -> PanelState {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_empty() {
		let panel = InventoryPanel::from(PanelState::Empty);
		assert_eq!(PanelState::Empty, panel.get(()));
	}

	#[test]
	fn get_filled() {
		let panel = InventoryPanel::from(PanelState::Filled);
		assert_eq!(PanelState::Filled, panel.get(()));
	}
}
