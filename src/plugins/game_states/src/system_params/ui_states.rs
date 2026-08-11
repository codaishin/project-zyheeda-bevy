use crate::states::ui::{ComboOverview, Hud, Inventory, Settings};
use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use common::traits::handles_game_states::UIState;

#[derive(SystemParam)]
pub(crate) struct UIStates<'w> {
	hud: ResMut<'w, NextState<Hud>>,
	inventory: ResMut<'w, NextState<Inventory>>,
	combos: ResMut<'w, NextState<ComboOverview>>,
	settings: ResMut<'w, NextState<Settings>>,
}

impl UIStates<'_> {
	pub(crate) fn set_on(&mut self, ui: UIState) {
		match ui {
			UIState::Hud => self.hud.set(Hud::On),
			UIState::Inventory => self.inventory.set(Inventory::On),
			UIState::ComboOverview => self.combos.set(ComboOverview::On),
			UIState::Settings => self.settings.set(Settings::On),
		}
	}

	pub(crate) fn set_off(&mut self, ui: &UIState) {
		match ui {
			UIState::Hud => self.hud.set(Hud::Off),
			UIState::Inventory => self.inventory.set(Inventory::Off),
			UIState::ComboOverview => self.combos.set(ComboOverview::Off),
			UIState::Settings => self.settings.set(Settings::Off),
		}
	}
}

impl UIStates<'static> {
	pub(crate) fn init(app: &mut App) {
		app.init_state::<Hud>();
		app.init_state::<Inventory>();
		app.init_state::<ComboOverview>();
		app.init_state::<Settings>();
	}

	pub(crate) fn on_enter<M>(
		app: &mut App,
		ui_state: UIState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match ui_state {
			UIState::Hud => app.add_systems(OnEnter(Hud::On), systems),
			UIState::Inventory => app.add_systems(OnEnter(Inventory::On), systems),
			UIState::ComboOverview => app.add_systems(OnEnter(ComboOverview::On), systems),
			UIState::Settings => app.add_systems(OnEnter(Settings::On), systems),
		};
	}

	pub(crate) fn on_exit<M>(
		app: &mut App,
		ui_state: UIState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match ui_state {
			UIState::Hud => app.add_systems(OnExit(Hud::On), systems),
			UIState::Inventory => app.add_systems(OnExit(Inventory::On), systems),
			UIState::ComboOverview => app.add_systems(OnExit(ComboOverview::On), systems),
			UIState::Settings => app.add_systems(OnExit(Settings::On), systems),
		};
	}
}
