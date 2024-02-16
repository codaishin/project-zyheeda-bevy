use crate::{components::QuickbarPanel, traits::get::Get};
use skills::components::SlotKey;

impl Get<(), SlotKey> for QuickbarPanel {
	fn get(&self, _: ()) -> SlotKey {
		self.key
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use common::components::Side;

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
