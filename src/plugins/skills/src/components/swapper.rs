use super::{inventory::Inventory, slots::Slots};
use crate::item::Item;
use bevy::prelude::*;
use bevy_rapier3d::na::max;
use common::{
	tools::{inventory_key::InventoryKey, slot_key::SlotKey},
	traits::handles_inventory_menu::SwapKeys,
};
use std::mem;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Swapper {
	swaps: Vec<(Key, Key)>,
}

#[derive(Debug, PartialEq)]
enum Key {
	Inventory(usize),
	Slot(SlotKey),
}

impl Swapper {
	pub(crate) fn system(mut swaps: Query<(&mut Self, &mut Inventory, &mut Slots)>) {
		for (mut swaps, mut inventory, mut slots) in &mut swaps {
			for swap in swaps.swaps.drain(..) {
				match swap {
					(Key::Inventory(a), Key::Inventory(b)) => {
						fill_until(&mut inventory, max(a, b));
						inventory.0.swap(a, b);
					}
					(Key::Slot(a), Key::Slot(b)) => {
						let item_a = slots.0.remove(&a).unwrap_or_default();
						let item_b = slots.0.remove(&b).unwrap_or_default();
						slots.0.insert(a, item_b);
						slots.0.insert(b, item_a);
					}
					(Key::Slot(s), Key::Inventory(i)) | (Key::Inventory(i), Key::Slot(s)) => {
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

impl SwapKeys<InventoryKey, InventoryKey> for Swapper {
	fn swap(&mut self, InventoryKey(a): InventoryKey, InventoryKey(b): InventoryKey) {
		self.swaps.push((Key::Inventory(a), Key::Inventory(b)));
	}
}

impl SwapKeys<InventoryKey, SlotKey> for Swapper {
	fn swap(&mut self, InventoryKey(a): InventoryKey, b: SlotKey) {
		self.swaps.push((Key::Inventory(a), Key::Slot(b)));
	}
}

impl SwapKeys<SlotKey, SlotKey> for Swapper {
	fn swap(&mut self, a: SlotKey, b: SlotKey) {
		self.swaps.push((Key::Slot(a), Key::Slot(b)));
	}
}

impl SwapKeys<SlotKey, InventoryKey> for Swapper {
	fn swap(&mut self, a: SlotKey, InventoryKey(b): InventoryKey) {
		self.swaps.push((Key::Slot(a), Key::Inventory(b)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::{new_handle, SingleThreadedApp},
		tools::slot_key::Side,
	};

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
				Inventory::new([Some(a.clone()), Some(b.clone())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(0), InventoryKey(1));
		app.update();

		assert_eq!(
			Some(&Inventory::new([Some(b), Some(a)])),
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
				Inventory::new([Some(item.clone())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(0), InventoryKey(3));
		app.update();

		assert_eq!(
			Some(&Inventory::new([None, None, None, Some(item)])),
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
				Inventory::new([Some(item.clone())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(3), InventoryKey(0));
		app.update();

		assert_eq!(
			Some(&Inventory::new([None, None, None, Some(item)])),
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
				Inventory::new([Some(item.clone()), None, None]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(0), InventoryKey(3));
		app.update();

		assert_eq!(
			Some(&Inventory::new([None, None, None, Some(item)])),
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
				Inventory::new([Some(item.clone()), None, None]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(0), InventoryKey(1));
		app.update();

		assert_eq!(
			Some(&Inventory::new([None, Some(item), None])),
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
				Slots::new([
					(SlotKey::BottomHand(Side::Left), Some(a.clone())),
					(SlotKey::BottomHand(Side::Right), Some(b.clone())),
				]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SlotKey::BottomHand(Side::Left),
				SlotKey::BottomHand(Side::Right),
			);
		app.update();

		assert_eq!(
			Some(&Slots::new([
				(SlotKey::BottomHand(Side::Left), Some(b)),
				(SlotKey::BottomHand(Side::Right), Some(a)),
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
				Slots::new([(SlotKey::BottomHand(Side::Left), Some(item.clone()))]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SlotKey::BottomHand(Side::Left),
				SlotKey::BottomHand(Side::Right),
			);
		app.update();

		assert_eq!(
			Some(&Slots::new([
				(SlotKey::BottomHand(Side::Left), None),
				(SlotKey::BottomHand(Side::Right), Some(item)),
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
				Slots::new([(SlotKey::BottomHand(Side::Left), Some(item.clone()))]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			);
		app.update();

		assert_eq!(
			Some(&Slots::new([
				(SlotKey::BottomHand(Side::Left), None),
				(SlotKey::BottomHand(Side::Right), Some(item)),
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
				Inventory::new([Some(a.clone())]),
				Slots::new([(SlotKey::BottomHand(Side::Left), Some(b.clone()))]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(SlotKey::BottomHand(Side::Left), InventoryKey(0));
		app.update();

		assert_eq!(
			(
				Some(&Inventory::new([Some(b)])),
				Some(&Slots::new([(SlotKey::BottomHand(Side::Left), Some(a)),]))
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
				Inventory::new([Some(item.clone())]),
				Slots::new([]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(SlotKey::BottomHand(Side::Left), InventoryKey(0));
		app.update();

		assert_eq!(
			(
				Some(&Inventory::new([None])),
				Some(&Slots::new(
					[(SlotKey::BottomHand(Side::Left), Some(item)),]
				))
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
				Inventory::new([]),
				Slots::new([(SlotKey::BottomHand(Side::Left), Some(item.clone()))]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(SlotKey::BottomHand(Side::Left), InventoryKey(0));
		app.update();

		assert_eq!(
			(
				Some(&Inventory::new([Some(item)])),
				Some(&Slots::new([(SlotKey::BottomHand(Side::Left), None)]))
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
				Inventory::new([Some(a.clone())]),
				Slots::new([(SlotKey::BottomHand(Side::Left), Some(b.clone()))]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(0), SlotKey::BottomHand(Side::Left));
		app.update();

		assert_eq!(
			(
				Some(&Inventory::new([Some(b)])),
				Some(&Slots::new([(SlotKey::BottomHand(Side::Left), Some(a)),]))
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
				Inventory::new([Some(new_handle()), Some(new_handle())]),
				Slots::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Swapper>()
			.unwrap()
			.swap(InventoryKey(0), InventoryKey(1));
		app.update();

		assert_eq!(
			Some(&Swapper::default()),
			app.world().entity(agent).get::<Swapper>(),
		);
	}
}
