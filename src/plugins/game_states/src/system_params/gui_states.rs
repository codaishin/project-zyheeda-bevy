use crate::states::gui::{ComboOverview, Hud, Inventory, Settings};
use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use common::prelude::*;
use std::collections::HashSet;
use zyheeda_core::prelude::*;

#[derive(SystemParam)]
pub(crate) struct GuiStates<'w> {
	hud: Res<'w, State<Hud>>,
	inventory: Res<'w, State<Inventory>>,
	combos: Res<'w, State<ComboOverview>>,
	settings: Res<'w, State<Settings>>,
}

impl GuiStates<'_> {
	pub(crate) fn is_on(&self, state: &Gui) -> bool {
		match state {
			Gui::Hud => self.hud.get() == &Hud::On,
			Gui::Inventory => self.inventory.get() == &Inventory::On,
			Gui::ComboOverview => self.combos.get() == &ComboOverview::On,
			Gui::Settings => self.settings.get() == &Settings::On,
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

impl GuiStates<'static> {
	pub(crate) fn init(app: &mut App) {
		app.init_state::<Hud>();
		app.init_state::<Inventory>();
		app.init_state::<ComboOverview>();
		app.init_state::<Settings>();
	}

	pub(crate) fn on_enter<M>(
		app: &mut App,
		ui_state: Gui,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match ui_state {
			Gui::Hud => app.add_systems(OnEnter(Hud::On), systems),
			Gui::Inventory => app.add_systems(OnEnter(Inventory::On), systems),
			Gui::ComboOverview => app.add_systems(OnEnter(ComboOverview::On), systems),
			Gui::Settings => app.add_systems(OnEnter(Settings::On), systems),
		};
	}

	pub(crate) fn on_exit<M>(
		app: &mut App,
		ui_state: Gui,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match ui_state {
			Gui::Hud => app.add_systems(OnExit(Hud::On), systems),
			Gui::Inventory => app.add_systems(OnExit(Inventory::On), systems),
			Gui::ComboOverview => app.add_systems(OnExit(ComboOverview::On), systems),
			Gui::Settings => app.add_systems(OnExit(Settings::On), systems),
		};
	}
}

impl From<&GuiStates<'_>> for HashSet<Gui> {
	fn from(ui: &GuiStates<'_>) -> Self {
		HashSet::from_iter(Gui::iterator().filter(|s| ui.is_on(s)))
	}
}

#[derive(SystemParam)]
pub(crate) struct GuiStatesMut<'w> {
	hud: ResMut<'w, NextState<Hud>>,
	inventory: ResMut<'w, NextState<Inventory>>,
	combos: ResMut<'w, NextState<ComboOverview>>,
	settings: ResMut<'w, NextState<Settings>>,
}

impl GuiStatesMut<'_> {
	pub(crate) fn set_on(&mut self, ui: Gui) {
		match ui {
			Gui::Hud => self.hud.set(Hud::On),
			Gui::Inventory => self.inventory.set(Inventory::On),
			Gui::ComboOverview => self.combos.set(ComboOverview::On),
			Gui::Settings => self.settings.set(Settings::On),
		}
	}

	pub(crate) fn set_off(&mut self, ui: &Gui) {
		match ui {
			Gui::Hud => self.hud.set(Hud::Off),
			Gui::Inventory => self.inventory.set(Inventory::Off),
			Gui::ComboOverview => self.combos.set(ComboOverview::Off),
			Gui::Settings => self.settings.set(Settings::Off),
		}
	}
}
