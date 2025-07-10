use crate::{
	tools::PanelState,
	traits::colors::{ColorConfig, HasActiveColor, HasPanelColors, HasQueuedColor, PanelColors},
};
use bevy::{color::Color, ecs::component::Component};
use common::{tools::action_key::slot::SlotKey, traits::accessors::get::GetterRef};

#[derive(Component)]
pub struct QuickbarPanel {
	pub key: SlotKey,
	pub state: PanelState,
}

impl GetterRef<PanelState> for QuickbarPanel {
	fn get(&self) -> &PanelState {
		&self.state
	}
}

impl GetterRef<SlotKey> for QuickbarPanel {
	fn get(&self) -> &SlotKey {
		&self.key
	}
}

impl HasPanelColors for QuickbarPanel {
	const PANEL_COLORS: PanelColors = PanelColors {
		disabled: PanelColors::DEFAULT.disabled,
		pressed: ColorConfig {
			background: Color::srgb(1., 0.27, 0.1),
			text: Color::srgb(0.9, 0.9, 0.9),
		},
		hovered: PanelColors::DEFAULT.filled,
		empty: PanelColors::DEFAULT.empty,
		filled: PanelColors::DEFAULT.filled,
	};
}

impl HasActiveColor for QuickbarPanel {
	const ACTIVE_COLORS: ColorConfig = ColorConfig {
		background: Color::srgb(0., 1., 0.),
		text: Color::srgb(0.9, 0.9, 0.9),
	};
}

impl HasQueuedColor for QuickbarPanel {
	const QUEUED_COLORS: ColorConfig = ColorConfig {
		background: Color::srgb(0., 1., 1.),
		text: Color::srgb(0.9, 0.9, 0.9),
	};
}

#[cfg(test)]
mod tests {
	use common::tools::action_key::slot::Side;

	use super::*;

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
