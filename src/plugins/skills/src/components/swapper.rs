use super::{inventory::Inventory, slots::Slots};
use crate::item::Item;
use bevy::prelude::*;
use bevy_rapier3d::na::max;
use common::{
	tools::{action_key::slot::SlotKey, inventory_key::InventoryKey, swap_key::SwapKey},
	traits::handles_loadout_menu::SwapValuesByKey,
};
use std::mem;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Swapper {
	swaps: Vec<(SwapKey, SwapKey)>,
}

impl Swapper {
	pub(crate) fn system(mut swaps: Query<(&mut Self, &mut Inventory, &mut Slots)>) {
		for (mut swaps, mut inventory, mut slots) in &mut swaps {
			for swap in swaps.swaps.drain(..) {
				match swap {
					(SwapKey::Inventory(InventoryKey(a)), SwapKey::Inventory(InventoryKey(b))) => {
						fill_until(&mut inventory, max(a, b));
						inventory.0.swap(a, b);
					}
					(SwapKey::Slot(a), SwapKey::Slot(b)) => {
						let item_a = slots.0.remove(&a).unwrap_or_default();
						let item_b = slots.0.remove(&b).unwrap_or_default();
						slots.0.insert(a, item_b);
						slots.0.insert(b, item_a);
					}
					(SwapKey::Slot(s), SwapKey::Inventory(InventoryKey(i)))
					| (SwapKey::Inventory(InventoryKey(i)), SwapKey::Slot(s)) => {
						let item_a = get_or_default_mut(&mut slots, s);
						let item_b = get_or_fill_mut(&mut inventory, i);
						mem::swap(item_a, item_b);
					}
				}
			}
		}
	}
}

fn get_or_default_mut(slots: &mut Slots, s: SlotKey) -> &mut Option<Handle<Item>> {
	slots.0.entry(s).or_default()
}

fn get_or_fill_mut(inventory: &mut Inventory, i: usize) -> &mut Option<Handle<Item>> {
	fill_until(inventory, i);
	&mut inventory.0[i]
}

fn fill_until(inventory: &mut Inventory, i: usize) {
	if inventory.0.len() > i {
		return;
	}

	inventory.0.resize(i + 1, None);
}

impl SwapValuesByKey for Swapper {
	fn swap(&mut self, a: SwapKey, b: SwapKey) {
		self.swaps.push((a, b));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Swapper::system);

		app
	}

	#[test]
	fn swap_inventory_items() {
		let a = new_handle();
		let b = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(a.clone()), Some(b.clone())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(0)),
				SwapKey::Inventory(InventoryKey(1)),
			);
		app.update();

		assert_eq!(
			Some(&Inventory::from([Some(b), Some(a)])),
			app.world().entity(agent).get::<Inventory>(),
		);
	}

	#[test]
	fn expand_inventory_before_swapping_out_of_bound_index() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(item.clone())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(0)),
				SwapKey::Inventory(InventoryKey(3)),
			);
		app.update();

		assert_eq!(
			Some(&Inventory::from([None, None, None, Some(item)])),
			app.world().entity(agent).get::<Inventory>(),
		);
	}

	#[test]
	fn expand_inventory_before_swapping_out_of_bound_index_reversed() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(item.clone())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(3)),
				SwapKey::Inventory(InventoryKey(0)),
			);
		app.update();

		assert_eq!(
			Some(&Inventory::from([None, None, None, Some(item)])),
			app.world().entity(agent).get::<Inventory>(),
		);
	}

	#[test]
	fn expand_inventory_before_swapping_index_of_inventory_length() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(item.clone()), None, None]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(0)),
				SwapKey::Inventory(InventoryKey(3)),
			);
		app.update();

		assert_eq!(
			Some(&Inventory::from([None, None, None, Some(item)])),
			app.world().entity(agent).get::<Inventory>(),
		);
	}

	#[test]
	fn do_not_shorten_inventory() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(item.clone()), None, None]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(0)),
				SwapKey::Inventory(InventoryKey(1)),
			);
		app.update();

		assert_eq!(
			Some(&Inventory::from([None, Some(item), None])),
			app.world().entity(agent).get::<Inventory>(),
		);
	}

	#[test]
	fn swap_slot_items() {
		let a = new_handle();
		let b = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::default(),
				Slots::from([
					(
						SlotKey::from(PlayerSlot::Lower(Side::Left)),
						Some(a.clone()),
					),
					(
						SlotKey::from(PlayerSlot::Lower(Side::Right)),
						Some(b.clone()),
					),
				]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Right))),
			);
		app.update();

		assert_eq!(
			Some(&Slots::from([
				(SlotKey::from(PlayerSlot::Lower(Side::Left)), Some(b)),
				(SlotKey::from(PlayerSlot::Lower(Side::Right)), Some(a)),
			])),
			app.world().entity(agent).get::<Slots>(),
		);
	}

	#[test]
	fn swap_slot_items_when_slot_not_set() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::default(),
				Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(item.clone()),
				)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Right))),
			);
		app.update();

		assert_eq!(
			Some(&Slots::from([
				(SlotKey::from(PlayerSlot::Lower(Side::Left)), None),
				(SlotKey::from(PlayerSlot::Lower(Side::Right)), Some(item)),
			])),
			app.world().entity(agent).get::<Slots>(),
		);
	}

	#[test]
	fn swap_slot_items_when_slot_not_set_reversed() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::default(),
				Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(item.clone()),
				)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Right))),
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
			);
		app.update();

		assert_eq!(
			Some(&Slots::from([
				(SlotKey::from(PlayerSlot::Lower(Side::Left)), None),
				(SlotKey::from(PlayerSlot::Lower(Side::Right)), Some(item)),
			])),
			app.world().entity(agent).get::<Slots>(),
		);
	}

	#[test]
	fn swap_slot_and_inventory_items() {
		let a = new_handle();
		let b = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(a.clone())]),
				Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(b.clone()),
				)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
				SwapKey::Inventory(InventoryKey(0)),
			);
		app.update();

		assert_eq!(
			(
				Some(&Inventory::from([Some(b)])),
				Some(&Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(a)
				),]))
			),
			(
				app.world().entity(agent).get::<Inventory>(),
				app.world().entity(agent).get::<Slots>(),
			)
		);
	}

	#[test]
	fn swap_slot_and_inventory_items_when_slot_not_set() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(item.clone())]),
				Slots::from([]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
				SwapKey::Inventory(InventoryKey(0)),
			);
		app.update();

		assert_eq!(
			(
				Some(&Inventory::from([None])),
				Some(&Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(item)
				),]))
			),
			(
				app.world().entity(agent).get::<Inventory>(),
				app.world().entity(agent).get::<Slots>(),
			)
		);
	}

	#[test]
	fn swap_slot_and_inventory_items_when_inventory_index_out_of_bounds() {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([]),
				Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(item.clone()),
				)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
				SwapKey::Inventory(InventoryKey(0)),
			);
		app.update();

		assert_eq!(
			(
				Some(&Inventory::from([Some(item)])),
				Some(&Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					None
				)]))
			),
			(
				app.world().entity(agent).get::<Inventory>(),
				app.world().entity(agent).get::<Slots>(),
			)
		);
	}

	#[test]
	fn swap_slot_and_inventory_items_reversed() {
		let a = new_handle();
		let b = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(a.clone())]),
				Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(b.clone()),
				)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(0)),
				SwapKey::Slot(SlotKey::from(PlayerSlot::Lower(Side::Left))),
			);
		app.update();

		assert_eq!(
			(
				Some(&Inventory::from([Some(b)])),
				Some(&Slots::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
					Some(a)
				),]))
			),
			(
				app.world().entity(agent).get::<Inventory>(),
				app.world().entity(agent).get::<Slots>(),
			)
		);
	}

	#[test]
	fn drain_swaps_from_swapper() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Swapper::default(),
				Inventory::from([Some(new_handle()), Some(new_handle())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SwapKey::Inventory(InventoryKey(0)),
				SwapKey::Inventory(InventoryKey(1)),
			);
		app.update();

		assert_eq!(
			Some(&Swapper::default()),
			app.world().entity(agent).get::<Swapper>(),
		);
	}
}
