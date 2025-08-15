use crate::{
	tools::PanelState,
	traits::colors::{ColorConfig, HasActiveColor, HasPanelColors, HasQueuedColor, PanelColors},
};
use bevy::{color::Color, ecs::component::Component};
use common::tools::action_key::slot::{PlayerSlot, SlotKey};

#[derive(Component)]
pub struct QuickbarPanel {
	pub key: PlayerSlot,
	pub state: PanelState,
}

impl From<&QuickbarPanel> for PanelState {
	fn from(QuickbarPanel { state, .. }: &QuickbarPanel) -> Self {
		*state
	}
}

impl From<&QuickbarPanel> for PlayerSlot {
	fn from(QuickbarPanel { key, .. }: &QuickbarPanel) -> Self {
		*key
	}
}

impl From<&QuickbarPanel> for SlotKey {
	fn from(QuickbarPanel { key, .. }: &QuickbarPanel) -> Self {
		Self::from(*key)
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
	use super::*;
	use common::{tools::action_key::slot::Side, traits::accessors::get::Getter};

	#[test]
	fn get_empty() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Right),
			state: PanelState::Empty,
		};
		assert_eq!(PanelState::Empty, panel.get::<PanelState>());
	}

	#[test]
	fn get_filled() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Right),
			state: PanelState::Filled,
		};
		assert_eq!(PanelState::Filled, panel.get::<PanelState>());
	}

	#[test]
	fn get_player_slot() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Left),
			state: PanelState::Empty,
		};

		assert_eq!(PlayerSlot::Lower(Side::Left), panel.get::<PlayerSlot>());
	}

	#[test]
	fn get_slot_key() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Left),
			state: PanelState::Empty,
		};

		assert_eq!(
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
			panel.get::<SlotKey>()
		);
	}
}
