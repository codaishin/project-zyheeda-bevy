use crate::components::{Collection, Inventory, InventoryKey, Item, Swap};
use bevy::{
	ecs::system::{Commands, Query},
	prelude::{Entity, Mut},
};
use std::cmp::max;

type ItemsToSwap<'a> = (
	Entity,
	&'a mut Inventory,
	&'a Collection<Swap<InventoryKey, InventoryKey>>,
);

pub fn swap_inventory_items(mut commands: Commands, mut items_to_swap: Query<ItemsToSwap>) {
	for (agent, mut inventory, swaps) in &mut items_to_swap {
		for swap in &swaps.0 {
			do_swap(&mut inventory, swap);
		}

		let mut agent = commands.entity(agent);
		agent.remove::<Collection<Swap<InventoryKey, InventoryKey>>>();
	}
}

fn do_swap(inventory: &mut Mut<Collection<Option<Item>>>, swap: &Swap<InventoryKey, InventoryKey>) {
	fill_to(&mut inventory.0, max(swap.0 .0, swap.1 .0));
	inventory.0.swap(swap.0 .0, swap.1 .0);
}

fn fill_to(inventory: &mut Vec<Option<Item>>, index: usize) {
	if index < inventory.len() {
		return;
	}

	inventory.extend(vec![None; index - inventory.len() + 1]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Item;
	use bevy::{
		app::{App, Update},
		prelude::default,
	};

	#[test]
	fn swap_items() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Inventory::new([
					Some(Item {
						name: "item a",
						..default()
					}),
					None,
					Some(Item {
						name: "item b",
						..default()
					}),
				]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);
		let inventory = agent.get::<Inventory>().unwrap();

		assert_eq!(
			vec![
				Some(Item {
					name: "item b",
					..default()
				}),
				None,
				Some(Item {
					name: "item a",
					..default()
				})
			],
			inventory.0
		)
	}

	#[test]
	fn swap_items_out_or_range() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Inventory::new([Some(Item {
					name: "item",
					..default()
				})]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);
		let inventory = agent.get::<Inventory>().unwrap();

		assert_eq!(
			vec![
				None,
				None,
				Some(Item {
					name: "item",
					..default()
				})
			],
			inventory.0
		)
	}

	#[test]
	fn swap_items_index_and_len_are_same() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Inventory::new([Some(Item {
					name: "item",
					..default()
				})]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(1))]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);
		let inventory = agent.get::<Inventory>().unwrap();

		assert_eq!(
			vec![
				None,
				Some(Item {
					name: "item",
					..default()
				})
			],
			inventory.0
		)
	}

	#[test]
	fn remove_swap_collection() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				Inventory::new([]),
				Collection::<Swap<InventoryKey, InventoryKey>>::new([]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<Swap<InventoryKey, InventoryKey>>>());
	}
}