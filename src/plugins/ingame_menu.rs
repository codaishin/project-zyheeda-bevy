mod components;
mod systems;
mod tools;
mod traits;

use self::{
	components::{InventoryPanel, InventoryScreen},
	systems::{
		colors::panel_color,
		dad::{drag::drag, drop::drop},
		despawn::despawn,
		set_state::set_state,
		spawn::spawn,
		toggle_state::toggle_state,
	},
	tools::{MenuState, PanelState},
};
use crate::{
	components::{DadPanel, Inventory, InventoryKey, Player, SlotKey, Slots, Swap},
	states::{GameRunning, Off, On},
};
use bevy::prelude::*;

pub struct IngameMenuPlugin;

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_state::<MenuState>()
			.add_systems(Update, toggle_state::<MenuState, Inventory>)
			.add_systems(
				OnEnter(MenuState::Inventory),
				(spawn::<InventoryScreen>, set_state::<GameRunning, Off>),
			)
			.add_systems(
				OnExit(MenuState::Inventory),
				(despawn::<InventoryScreen>, set_state::<GameRunning, On>),
			)
			.add_systems(
				Update,
				(
					panel_color::<InventoryPanel>,
					drag::<Player, InventoryKey>,
					drag::<Player, SlotKey>,
					drop::<Player, InventoryKey, InventoryKey, Swap<InventoryKey, InventoryKey>>,
					drop::<Player, SlotKey, SlotKey, Swap<SlotKey, SlotKey>>,
					drop::<Player, SlotKey, InventoryKey, Swap<SlotKey, InventoryKey>>,
					drop::<Player, InventoryKey, SlotKey, Swap<InventoryKey, SlotKey>>,
					update_slots,
					update_item,
				)
					.run_if(in_state(MenuState::Inventory)),
			);
	}
}

fn update_slots(
	slots: Query<&Slots>,
	agents: Query<Entity, With<Player>>,
	mut texts: Query<&mut Text>,
	mut slot_buttons: Query<(&DadPanel<SlotKey>, &Children, &mut InventoryPanel), With<Button>>,
) {
	let player = agents.single();

	for (target_panel, children, mut panel) in &mut slot_buttons {
		let mut txt = texts.get_mut(children[0]).unwrap();
		let slots = slots.get(player).unwrap();
		match slots.0.get(&target_panel.0).and_then(|s| s.item) {
			Some(item) => {
				txt.sections[0].value = item.name.to_string();
				*panel = PanelState::Filled.into();
			}
			_ => {
				txt.sections[0].value = "<Empty>".to_string();
				*panel = PanelState::Empty.into();
			}
		};
	}
}

fn update_item(
	inventory: Query<&Inventory>,
	agents: Query<Entity, With<Player>>,
	mut texts: Query<&mut Text>,
	mut inventory_buttons: Query<
		(&DadPanel<InventoryKey>, &Children, &mut InventoryPanel),
		With<Button>,
	>,
) {
	let player = agents.single();

	for (target_panel, children, mut panel) in &mut inventory_buttons {
		let mut txt = texts.get_mut(children[0]).unwrap();
		let inventory = inventory.get(player).unwrap();
		match inventory.0.get(target_panel.0 .0) {
			Some(Some(item)) => {
				txt.sections[0].value = item.name.to_string();
				*panel = PanelState::Filled.into();
			}
			_ => {
				txt.sections[0].value = "<Empty>".to_string();
				*panel = PanelState::Empty.into();
			}
		};
	}
}
