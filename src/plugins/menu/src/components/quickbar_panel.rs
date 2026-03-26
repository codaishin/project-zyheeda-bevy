use crate::{
	tools::PanelState,
	traits::colors::{ColorConfig, HasPanelColors, PanelColors},
};
use bevy::{color::Color, ecs::component::Component};
use common::{
	tools::action_key::slot::{PlayerSlot, SlotKey},
	traits::accessors::get::View,
};

#[derive(Component)]
pub struct QuickbarPanel {
	pub key: PlayerSlot,
	pub state: PanelState,
}

impl QuickbarPanel {
	pub(crate) const PANEL_COLORS: PanelColors = PanelColors {
		disabled: PanelColors::DEFAULT.disabled,
		pressed: ColorConfig {
			background: Color::srgb(1., 0.27, 0.1),
			text: Color::srgb(0.9, 0.9, 0.9),
		},
		hovered: PanelColors::DEFAULT.filled,
		empty: PanelColors::DEFAULT.empty,
		filled: PanelColors::DEFAULT.filled,
	};
	pub(crate) const ACTIVE_COLORS: ColorConfig = ColorConfig {
		background: Color::srgb(0., 1., 0.),
		text: Color::srgb(0.9, 0.9, 0.9),
	};
	pub(crate) const QUEUED_COLORS: ColorConfig = ColorConfig {
		background: Color::srgb(0., 1., 1.),
		text: Color::srgb(0.9, 0.9, 0.9),
	};
}

impl From<PlayerSlot> for QuickbarPanel {
	fn from(key: PlayerSlot) -> Self {
		Self {
			key,
			state: PanelState::Empty,
		}
	}
}

impl View<PanelState> for QuickbarPanel {
	fn view(&self) -> PanelState {
		self.state
	}
}

impl View<PlayerSlot> for QuickbarPanel {
	fn view(&self) -> PlayerSlot {
		self.key
	}
}

impl View<SlotKey> for QuickbarPanel {
	fn view(&self) -> SlotKey {
		SlotKey::from(self.key)
	}
}

impl HasPanelColors for QuickbarPanel {
	const PANEL_COLORS: PanelColors = Self::PANEL_COLORS;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::action_key::slot::Side, traits::accessors::get::ViewOf};

	#[test]
	fn get_empty() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Right),
			state: PanelState::Empty,
		};
		assert_eq!(PanelState::Empty, panel.view_of::<PanelState>());
	}

	#[test]
	fn get_filled() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Right),
			state: PanelState::Filled,
		};
		assert_eq!(PanelState::Filled, panel.view_of::<PanelState>());
	}

	#[test]
	fn get_player_slot() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Left),
			state: PanelState::Empty,
		};

		assert_eq!(PlayerSlot::Lower(Side::Left), panel.view_of::<PlayerSlot>());
	}

	#[test]
	fn get_slot_key() {
		let panel = QuickbarPanel {
			key: PlayerSlot::Lower(Side::Left),
			state: PanelState::Empty,
		};

		assert_eq!(
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
			panel.view_of::<SlotKey>(),
		);
	}
}
