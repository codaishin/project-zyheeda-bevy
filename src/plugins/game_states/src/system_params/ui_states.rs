use crate::states::ui::{ComboOverview, Hud, Inventory, Settings};
use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use common::prelude::*;
use std::collections::HashSet;
use zyheeda_core::prelude::*;

#[derive(SystemParam)]
pub(crate) struct UIStates<'w> {
	hud: Res<'w, State<Hud>>,
	inventory: Res<'w, State<Inventory>>,
	combos: Res<'w, State<ComboOverview>>,
	settings: Res<'w, State<Settings>>,
}

impl UIStates<'_> {
	pub(crate) fn is_on(&self, state: &UIState) -> bool {
		match state {
			UIState::Hud => self.hud.get() == &Hud::On,
			UIState::Inventory => self.inventory.get() == &Inventory::On,
			UIState::ComboOverview => self.combos.get() == &ComboOverview::On,
			UIState::Settings => self.settings.get() == &Settings::On,
		}
	}

	pub(crate) fn is_changed(&self) -> bool {
		any!(is_changed(
			self.hud,
			self.inventory,
			self.combos,
			self.settings
		))
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

impl From<&UIStates<'_>> for HashSet<UIState> {
	fn from(ui: &UIStates<'_>) -> Self {
		HashSet::from_iter(UIState::iterator().filter(|s| ui.is_on(s)))
	}
}

#[derive(SystemParam)]
pub(crate) struct UIStatesMut<'w> {
	hud: ResMut<'w, NextState<Hud>>,
	inventory: ResMut<'w, NextState<Inventory>>,
	combos: ResMut<'w, NextState<ComboOverview>>,
	settings: ResMut<'w, NextState<Settings>>,
}

impl UIStatesMut<'_> {
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
