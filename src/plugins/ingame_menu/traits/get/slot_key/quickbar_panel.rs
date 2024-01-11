use crate::{
	components::SlotKey,
	plugins::ingame_menu::{components::QuickbarPanel, traits::get::Get},
};

impl Get<(), SlotKey> for QuickbarPanel {
	fn get(&self, _: ()) -> SlotKey {
		self.key
	}
}

#[cfg(test)]
mod tests {
	use crate::{components::Side, plugins::ingame_menu::tools::PanelState};

	use super::*;

	#[test]
	fn get_legs() {
		let panel = QuickbarPanel {
			key: SlotKey::Legs,
			state: PanelState::Empty,
		};

		assert_eq!(SlotKey::Legs, panel.get(()));
	}

	#[test]
	fn get_main_hand() {
		let panel = QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		};

		assert_eq!(SlotKey::Hand(Side::Main), panel.get(()));
	}
}
