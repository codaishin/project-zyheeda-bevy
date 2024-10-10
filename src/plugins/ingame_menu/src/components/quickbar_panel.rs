use crate::{
	tools::PanelState,
	traits::colors::{
		HasActiveColor,
		HasPanelColors,
		HasQueuedColor,
		PanelColors,
		DEFAULT_PANEL_COLORS,
	},
};
use bevy::{color::Color, ecs::component::Component};
use common::traits::get::GetStatic;
use skills::items::slot_key::SlotKey;

#[derive(Component)]
pub struct QuickbarPanel {
	pub key: SlotKey,
	pub state: PanelState,
}

impl GetStatic<PanelState> for QuickbarPanel {
	fn get(&self) -> &PanelState {
		&self.state
	}
}

impl GetStatic<SlotKey> for QuickbarPanel {
	fn get(&self) -> &SlotKey {
		&self.key
	}
}

impl HasPanelColors for QuickbarPanel {
	const PANEL_COLORS: PanelColors = PanelColors {
		pressed: Color::srgb(1., 0.27, 0.1),
		hovered: DEFAULT_PANEL_COLORS.filled,
		empty: DEFAULT_PANEL_COLORS.empty,
		filled: DEFAULT_PANEL_COLORS.filled,
		text: DEFAULT_PANEL_COLORS.text,
	};
}

impl HasActiveColor for QuickbarPanel {
	const ACTIVE_COLOR: Color = Color::srgb(0., 1., 0.);
}

impl HasQueuedColor for QuickbarPanel {
	const QUEUED_COLOR: Color = Color::srgb(0., 1., 1.);
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::components::Side;
	use skills::items::slot_key::SlotKey;

	#[test]
	fn get_empty() {
		let panel = QuickbarPanel {
			key: SlotKey::BottomHand(Side::Right),
			state: PanelState::Empty,
		};
		assert_eq!(&PanelState::Empty, panel.get());
	}

	#[test]
	fn get_filled() {
		let panel = QuickbarPanel {
			key: SlotKey::BottomHand(Side::Right),
			state: PanelState::Filled,
		};
		assert_eq!(&PanelState::Filled, panel.get());
	}

	#[test]
	fn get_legs() {
		let panel = QuickbarPanel {
			key: SlotKey::BottomHand(Side::Left),
			state: PanelState::Empty,
		};

		assert_eq!(&SlotKey::BottomHand(Side::Left), panel.get());
	}

	#[test]
	fn get_main_hand() {
		let panel = QuickbarPanel {
			key: SlotKey::BottomHand(Side::Right),
			state: PanelState::Empty,
		};

		assert_eq!(&SlotKey::BottomHand(Side::Right), panel.get());
	}
}
