mod components;
mod systems;
mod tools;
mod traits;

use self::{
	components::{InventoryPanel, InventoryScreen},
	systems::{
		colors::panel_color,
		despawn::despawn,
		set_state::set_state,
		spawn::spawn,
		toggle_state::toggle_state,
	},
	tools::{MenuState, PanelState},
};
use crate::{
	components::{Collection, Inventory, InventoryKey, Player, SlotKey, Slots, Swap, TargetPanel},
	states::{GameRunning, Off, On},
};
use bevy::prelude::*;

pub struct IngameMenuPlugin;

#[derive(Component, Debug)]
struct Drag<T> {
	pub key: T,
}

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
					drag_and_drop,
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
	mut slot_buttons: Query<(&TargetPanel<SlotKey>, &Children, &mut InventoryPanel), With<Button>>,
) {
	let player = agents.single();

	for (target_panel, children, mut panel) in &mut slot_buttons {
		let mut txt = texts.get_mut(children[0]).unwrap();
		let slots = slots.get(player).unwrap();
		match slots.0.get(&target_panel.key).and_then(|s| s.item) {
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
		(&TargetPanel<InventoryKey>, &Children, &mut InventoryPanel),
		With<Button>,
	>,
) {
	let player = agents.single();

	for (target_panel, children, mut panel) in &mut inventory_buttons {
		let mut txt = texts.get_mut(children[0]).unwrap();
		let inventory = inventory.get(player).unwrap();
		match inventory.0.get(target_panel.key.0) {
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

type Panels<'a> = (
	&'a Interaction,
	Option<&'a TargetPanel<InventoryKey>>,
	Option<&'a TargetPanel<SlotKey>>,
);

fn drag_and_drop(
	mut commands: Commands,
	mouse: Res<Input<MouseButton>>,
	agents: Query<Entity, With<Player>>,
	drag_from_inventory: Query<&Drag<InventoryKey>>,
	drag_from_equipment: Query<&Drag<SlotKey>>,
	mut panels: Query<Panels, With<Button>>,
) {
	let player = agents.single();

	for (interaction, inv_p, equ_p) in &mut panels {
		let mouse_pressed = mouse.just_pressed(MouseButton::Left);
		let mouse_released = mouse.just_released(MouseButton::Left);
		let inv_d = drag_from_inventory.get(player);
		let equ_d = drag_from_equipment.get(player);
		match (interaction, mouse_pressed, mouse_released) {
			(Interaction::Pressed, true, false) => match (inv_p, equ_p) {
				(Some(panel), _) => {
					let key = panel.key;
					commands.entity(player).insert(Drag { key });
				}
				(_, Some(panel)) => {
					let key = panel.key;
					commands.entity(player).insert(Drag { key });
				}
				_ => {
					println!("PRESSING INCOMPATIBLE BUTTONS, SHOULD NOT HAVE HAPPENED")
				}
			},
			(Interaction::Hovered, false, true) => match (inv_p, inv_d, equ_p, equ_d) {
				(Some(inv_p), _, _, Ok(equ_d)) => {
					commands
						.entity(player)
						.insert(Collection::new([Swap(inv_p.key, equ_d.key)]));
					commands.entity(player).remove::<Drag<SlotKey>>();
				}
				(_, _, Some(equ_p), Ok(equ_d)) => {
					commands
						.entity(player)
						.insert(Collection::new([Swap(equ_p.key, equ_d.key)]));
					commands.entity(player).remove::<Drag<SlotKey>>();
				}
				(_, Ok(inv_d), Some(equ_p), _) => {
					commands
						.entity(player)
						.insert(Collection::new([Swap(inv_d.key, equ_p.key)]));
					commands.entity(player).remove::<Drag<usize>>();
				}
				(Some(inv_p), Ok(inv_d), ..) => {
					commands
						.entity(player)
						.insert(Collection::new([Swap(inv_p.key, inv_d.key)]));
					commands.entity(player).remove::<Drag<usize>>();
				}
				combination => {
					println!(
						"DRAGGING INCOMPATIBLE BUTTONS {:?} (SHOULD NOT HAPPEN)",
						combination
					);
				}
			},
			_ => {}
		}
	}
}
