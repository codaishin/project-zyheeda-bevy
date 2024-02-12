use crate::{components::InventoryPanel, tools::PanelState};

impl From<PanelState> for InventoryPanel {
	fn from(value: PanelState) -> Self {
		Self(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_filled() {
		assert_eq!(
			InventoryPanel(PanelState::Filled),
			InventoryPanel::from(PanelState::Filled)
		);
	}

	#[test]
	fn from_empty() {
		assert_eq!(
			InventoryPanel(PanelState::Empty),
			InventoryPanel::from(PanelState::Empty)
		);
	}
}
