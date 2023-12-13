mod components;
mod systems;
mod tools;
mod traits;

use self::{
	components::{InventoryPanel, InventoryScreen},
	systems::{colors::panel_color, despawn::despawn, set_state::set_state, spawn::spawn},
	tools::PanelState,
};
use crate::{
	components::{
		Collection,
		Inventory,
		InventoryKey,
		Player,
		Side,
		Slot,
		SlotKey,
		Slots,
		Swap,
		TargetPanel,
	},
	states::{GameRunning, Off, On},
};
use bevy::prelude::*;

const EQUIPMENT_SLOTS: [(SlotKey, &str); 2] = [
	(SlotKey::Hand(Side::Left), "Left Hand"),
	(SlotKey::Hand(Side::Right), "Right Hand"),
];

pub struct IngameMenuPlugin;

#[derive(Component, Debug)]
struct Drag<T> {
	pub value: T,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
	#[default]
	None,
	Inventory,
}

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_state::<MenuState>()
			.add_systems(Update, toggle_inventory)
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

fn toggle_inventory(
	keys: Res<Input<KeyCode>>,
	current_state: Res<State<MenuState>>,
	mut next_state: ResMut<NextState<MenuState>>,
) {
	if keys.just_pressed(KeyCode::I) {
		let state = match current_state.get() {
			MenuState::Inventory => MenuState::None,
			MenuState::None => MenuState::Inventory,
		};
		next_state.set(state);
	}
}

fn update_slots(
	slots: Query<&Slots>,
	agents: Query<Entity, With<Player>>,
	mut texts: Query<&mut Text>,
	mut slot_buttons: Query<(&TargetPanel<Slot>, &Children, &mut InventoryPanel), With<Button>>,
) {
	let player = agents.single();

	for (target_panel, children, mut panel) in &mut slot_buttons {
		let mut txt = texts.get_mut(children[0]).unwrap();
		let slots = slots.get(player).unwrap();
		let (slot_key, _) = EQUIPMENT_SLOTS[target_panel.index];
		match slots.0.get(&slot_key).and_then(|s| s.item) {
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
		(&TargetPanel<Inventory>, &Children, &mut InventoryPanel),
		With<Button>,
	>,
) {
	let player = agents.single();

	for (target_panel, children, mut panel) in &mut inventory_buttons {
		let mut txt = texts.get_mut(children[0]).unwrap();
		let inventory = inventory.get(player).unwrap();
		match inventory.0.get(target_panel.index) {
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
	Option<&'a TargetPanel<Inventory>>,
	Option<&'a TargetPanel<Slot>>,
);

fn drag_and_drop(
	mut commands: Commands,
	mouse: Res<Input<MouseButton>>,
	agents: Query<Entity, With<Player>>,
	drag_from_inventory: Query<&Drag<usize>>,
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
					let value = panel.index;
					commands.entity(player).insert(Drag { value });
				}
				(_, Some(panel)) => {
					let (value, _) = EQUIPMENT_SLOTS[panel.index];
					commands.entity(player).insert(Drag { value });
				}
				_ => {
					println!("PRESSING INCOMPATIBLE BUTTONS, SHOULD NOT HAVE HAPPENED")
				}
			},
			(Interaction::Hovered, false, true) => match (inv_p, inv_d, equ_p, equ_d) {
				(Some(inv_p), _, _, Ok(equ_d)) => {
					let inventory_key = inv_p.index;
					let slot_key = equ_d.value;
					commands.entity(player).insert(Collection::new([Swap(
						InventoryKey(inventory_key),
						slot_key,
					)]));
					commands.entity(player).remove::<Drag<SlotKey>>();
				}
				(_, _, Some(equ_p), Ok(equ_d)) => {
					let (slot_key, _) = EQUIPMENT_SLOTS[equ_p.index];
					commands
						.entity(player)
						.insert(Collection::new([Swap(slot_key, equ_d.value)]));
					commands.entity(player).remove::<Drag<SlotKey>>();
				}
				(_, Ok(inv_d), Some(equ_p), _) => {
					let (slot_key, _) = EQUIPMENT_SLOTS[equ_p.index];
					let inventory_key = inv_d.value;
					commands.entity(player).insert(Collection::new([Swap(
						InventoryKey(inventory_key),
						slot_key,
					)]));
					commands.entity(player).remove::<Drag<usize>>();
				}
				(Some(inv_p), Ok(inv_d), ..) => {
					commands.entity(player).insert(Collection::new([Swap(
						InventoryKey(inv_p.index),
						InventoryKey(inv_d.value),
					)]));
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
