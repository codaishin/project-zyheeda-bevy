use bevy::{
	asset::Handle,
	ecs::system::{Commands, Query},
	prelude::{Entity, Mut},
};
use common::{
	components::{Collection, Swap},
	traits::try_remove_from::TryRemoveFrom,
};
use skills::{
	components::inventory::Inventory,
	items::{inventory_key::InventoryKey, Item},
	skills::Skill,
};
use std::cmp::max;

type ItemsToSwap<'a> = (
	Entity,
	&'a mut Inventory<Handle<Skill>>,
	&'a Collection<Swap<InventoryKey, InventoryKey>>,
);

pub fn swap_inventory_items(mut commands: Commands, mut items_to_swap: Query<ItemsToSwap>) {
	for (agent, mut inventory, swaps) in &mut items_to_swap {
		for swap in &swaps.0 {
			do_swap(&mut inventory, swap);
		}

		commands.try_remove_from::<Collection<Swap<InventoryKey, InventoryKey>>>(agent);
	}
}

fn do_swap(
	inventory: &mut Mut<Collection<Option<Item<Handle<Skill>>>>>,
	swap: &Swap<InventoryKey, InventoryKey>,
) {
	fill_to(&mut inventory.0, max(swap.0 .0, swap.1 .0));
	inventory.0.swap(swap.0 .0, swap.1 .0);
}

fn fill_to(inventory: &mut Vec<Option<Item<Handle<Skill>>>>, index: usize) {
	if index < inventory.len() {
		return;
	}

	inventory.extend(vec![None; index - inventory.len() + 1]);
}

#[cfg(test)]
mod tests {
	use super::*;
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
				Inventory::<Handle<Skill>>::new([
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
		let inventory = agent.get::<Inventory<Handle<Skill>>>().unwrap();

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
				Inventory::<Handle<Skill>>::new([Some(Item {
					name: "item",
					..default()
				})]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);
		let inventory = agent.get::<Inventory<Handle<Skill>>>().unwrap();

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
				Inventory::<Handle<Skill>>::new([Some(Item {
					name: "item",
					..default()
				})]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(1))]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);
		let inventory = agent.get::<Inventory<Handle<Skill>>>().unwrap();

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
				Inventory::<Handle<Skill>>::new([]),
				Collection::<Swap<InventoryKey, InventoryKey>>::new([]),
			))
			.id();

		app.add_systems(Update, swap_inventory_items);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<Swap<InventoryKey, InventoryKey>>>());
	}
}
