use super::GetPanelState;
use crate::plugins::ingame_menu::{components::InventoryPanel, tools::PanelState};

impl GetPanelState for InventoryPanel {
	fn get_panel_state(&self) -> PanelState {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_empty() {
		let panel = InventoryPanel::from(PanelState::Empty);
		assert_eq!(PanelState::Empty, panel.get_panel_state());
	}

	#[test]
	fn get_filled() {
		let panel = InventoryPanel::from(PanelState::Filled);
		assert_eq!(PanelState::Filled, panel.get_panel_state());
	}
}
