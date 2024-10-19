use crate::{
	tools::PanelState,
	traits::{
		colors::{HasPanelColors, PanelColors, DEFAULT_PANEL_COLORS},
		set::Set,
	},
};
use bevy::ecs::component::Component;
use common::traits::get::GetterRef;

#[derive(Component, Debug, PartialEq)]
pub struct InventoryPanel(pub PanelState);

impl From<PanelState> for InventoryPanel {
	fn from(value: PanelState) -> Self {
		Self(value)
	}
}

impl GetterRef<PanelState> for InventoryPanel {
	fn get(&self) -> &PanelState {
		&self.0
	}
}

impl Set<(), PanelState> for InventoryPanel {
	fn set(&mut self, _: (), value: PanelState) {
		self.0 = value;
	}
}

impl HasPanelColors for InventoryPanel {
	const PANEL_COLORS: PanelColors = DEFAULT_PANEL_COLORS;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_empty() {
		let panel = InventoryPanel::from(PanelState::Empty);
		assert_eq!(&PanelState::Empty, panel.get());
	}

	#[test]
	fn get_filled() {
		let panel = InventoryPanel::from(PanelState::Filled);
		assert_eq!(&PanelState::Filled, panel.get());
	}

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
