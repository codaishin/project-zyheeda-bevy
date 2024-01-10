use crate::plugins::ingame_menu::{components::QuickbarPanel, tools::PanelState, traits::get::Get};

impl Get<(), PanelState> for QuickbarPanel {
	fn get(&self, _: ()) -> PanelState {
		self.state
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SlotKey;

	#[test]
	fn get_empty() {
		let panel = QuickbarPanel {
			key: SlotKey::Legs,
			state: PanelState::Empty,
		};
		assert_eq!(PanelState::Empty, panel.get(()));
	}

	#[test]
	fn get_filled() {
		let panel = QuickbarPanel {
			key: SlotKey::Legs,
			state: PanelState::Filled,
		};
		assert_eq!(PanelState::Filled, panel.get(()));
	}
}
