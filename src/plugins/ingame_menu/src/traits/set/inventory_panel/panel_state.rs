use crate::{components::InventoryPanel, tools::PanelState, traits::set::Set};

impl Set<(), PanelState> for InventoryPanel {
	fn set(&mut self, _: (), value: PanelState) {
		self.0 = value;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_empty() {
		let mut panel = InventoryPanel::from(PanelState::Filled);
		panel.set((), PanelState::Empty);
		assert_eq!(PanelState::Empty, panel.0);
	}

	#[test]
	fn set_filled() {
		let mut panel = InventoryPanel::from(PanelState::Empty);
		panel.set((), PanelState::Filled);
		assert_eq!(PanelState::Filled, panel.0);
	}
}
