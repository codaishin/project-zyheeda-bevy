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
use bevy::{ecs::component::Component, render::color::Color};
use common::traits::get::GetStatic;
use skills::items::SlotKey;

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
		pressed: Color::ORANGE_RED,
		hovered: DEFAULT_PANEL_COLORS.filled,
		empty: DEFAULT_PANEL_COLORS.empty,
		filled: DEFAULT_PANEL_COLORS.filled,
		text: DEFAULT_PANEL_COLORS.text,
	};
}

impl HasActiveColor for QuickbarPanel {
	const ACTIVE_COLOR: Color = Color::GREEN;
}

impl HasQueuedColor for QuickbarPanel {
	const QUEUED_COLOR: Color = Color::YELLOW_GREEN;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::components::Side;
	use skills::items::SlotKey;

	#[test]
	fn get_empty() {
		let panel = QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		};
		assert_eq!(&PanelState::Empty, panel.get());
	}

	#[test]
	fn get_filled() {
		let panel = QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Filled,
		};
		assert_eq!(&PanelState::Filled, panel.get());
	}

	#[test]
	fn get_legs() {
		let panel = QuickbarPanel {
			key: SlotKey::Hand(Side::Off),
			state: PanelState::Empty,
		};

		assert_eq!(&SlotKey::Hand(Side::Off), panel.get());
	}

	#[test]
	fn get_main_hand() {
		let panel = QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		};

		assert_eq!(&SlotKey::Hand(Side::Main), panel.get());
	}
}
